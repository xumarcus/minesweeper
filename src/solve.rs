// Copyright (C) 2021 Marcus Xu
//
// This file is part of minesweeper.
//
// minesweeper is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// minesweeper is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with minesweeper.  If not, see <http://www.gnu.org/licenses/>.

use super::*;

#[derive(Clone, Debug)]
pub struct Solver {
    config: Config,
    squares: Vec<Square>,
}

impl Solver {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            squares: (0..config.size())
                .map(|idx| config.square(idx).collect())
                .collect(),
        }
    }

    pub fn size(&self) -> usize {
        self.squares.len()
    }

    pub fn square(&self, idx: Index) -> &[Index] {
        &self.squares[idx]
    }

    fn square_of<'a, F: 'static + Fn(Status) -> bool>(
        &'a self,
        state: &'a MinesweeperState,
        idx: Index,
        f: F,
    ) -> impl Iterator<Item = Index> + 'a {
        self.square(idx)
            .iter()
            .filter_map(move |&cidx| f(state.get(cidx)).then(|| cidx))
    }

    fn make_consistent(&self, idx: Index, state: &mut MinesweeperState) -> bool {
        state
            .get_known(idx)
            .map(|count| {
                let minimum = self
                    .square_of(state, idx, |status| status == Status::Flagged)
                    .count();
                let maximum = self
                    .square_of(state, idx, |status| status == Status::Unknown)
                    .count()
                    + minimum;
                if !(minimum..=maximum).contains(&count) {
                    return false;
                }
                let square = self
                    .square_of(state, idx, |status| status == Status::Unknown)
                    .collect::<Square>();
                square.into_iter().all(|cidx| {
                    if count == minimum && state.get(cidx) != Status::Marked {
                        if !state.set_mark(cidx) {
                            return false;
                        }
                    } else if count == maximum && state.get(cidx) != Status::Flagged {
                        if !state.set_flag(cidx) {
                            return false;
                        }
                    } else {
                        return true;
                    }
                    self.make_consistent_sq(cidx, state)
                })
            })
            .unwrap_or(true)
    }

    fn make_consistent_sq(&self, idx: Index, state: &mut MinesweeperState) -> bool {
        self.square(idx)
            .iter()
            .all(|&cidx| self.make_consistent(cidx, state))
    }

    fn make_consistent_all(&self, state: &mut MinesweeperState) -> bool {
        (0..self.size()).all(|cidx| self.make_consistent(cidx, state))
    }

    fn branching_evaluation(&self, state: &MinesweeperState, group: &Group) -> Option<Evaluation> {
        let idx = group.get()?;
        let mut sf = state.clone();
        let mut sm = state.clone();

        // Short circuiting
        let sf = (sf.set_flag(idx) && self.make_consistent_sq(idx, &mut sf))
            .then(|| self.splitting_evaluation(&sf, group))
            .flatten();
        let sm = (sm.set_mark(idx) && self.make_consistent_sq(idx, &mut sm))
            .then(|| self.splitting_evaluation(&sm, group))
            .flatten();
        util::lift(Evaluation::add)(sf, sm)
    }

    fn splitting_evaluation(&self, state: &MinesweeperState, group: &Group) -> Option<Evaluation> {
        let (group, remainder) = group.trim(state);
        let eval = Evaluation::new(state, remainder);
        match group {
            Some(group) => group.into_iter().fold(Some(eval), |eval, split| {
                Some(eval? * self.branching_evaluation(state, &split)?)
            }),
            None => Some(eval),
        }
    }

    fn corner_search(&self, state: &MinesweeperState) -> Option<ScoredIndex> {
        match state.get(0) {
            Status::Known(_) => None,
            Status::Flagged | Status::Marked => unreachable!(),
            _ => Some((R64::new(0.0), 0)),
        }
    }

    fn fast_search(state: &MinesweeperState) -> Option<ScoredIndex> {
        let idx = state
            .board()
            .iter()
            .position(|status| status == &Status::Marked)?;
        Some((R64::new(0.0), idx))
    }

    fn solve_state(&self, state: &mut MinesweeperState) -> Option<ScoredIndex> {
        util::catch(move || {
            util::wrap(self.corner_search(state))?;
            util::wrap(Self::fast_search(state))?;
            util::guard(self.make_consistent_all(state))?;
            util::wrap(Self::fast_search(state))?;
            let (group, remainder) = Group::new(&self, state);
            let eval = util::guard_from(|| self.branching_evaluation(state, group.as_ref()?))?;
            eval.label(state);
            util::wrap(Self::fast_search(state))?;
            log::debug!("{:?}", eval);

            let mut v = vec![None; self.size()];
            let (bp, ps) = eval.to_probabilities(state.flags_remaining(), remainder.count_ones());
            for (p, idx) in ps {
                v[idx] = Some(p);
            }
            for idx in remainder.iter_ones() {
                v[idx] = bp;
            }
            util::wrap(v.iter()
                .enumerate()
                .filter_map(|(idx, p)| Some((*p.as_ref()?, idx)))
                .min_by_key(|(p, idx)| {
                    let p0 = self
                        .square(*idx)
                        .iter()
                        .filter_map(|&cidx| Some(R64::new(1.0) - v[cidx]?))
                        .product::<R64>();
                    *p * (R64::new(1.0) - p0)
                })
            )?;
            Ok(())
        })
    }

    pub fn solve_next<T: Minesweeper>(&self, sweep: &mut T) -> MsResult<Option<ScoredIndex>> {
        let mut state = sweep.pull()?;
        let scored_index = self.solve_state(&mut state);
        log::info!("{:?}", scored_index);
        let indices = sweep
            .get_state()
            .board()
            .iter()
            .zip(state.board().iter())
            .enumerate()
            .filter_map(|(idx, (p, n))| {
                (p != &Status::Flagged && n == &Status::Flagged).then(|| idx)
            })
            .collect::<Vec<Index>>();
        for idx in indices {
            sweep.flag(idx)?;
        }
        sweep.set_internal(state)?;
        if let Some((_, idx)) = scored_index {
            sweep.reveal(idx)?;
        }
        Ok(scored_index)
    }

    pub fn solve<T: Minesweeper>(&self, sweep: &mut T) -> MsResult<()> {
        while let Some(x) = self.solve_next(sweep)? {
            drop(x);
        }
        Ok(())
    }
}
