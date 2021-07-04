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

use show::ShowState;

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

    fn evaluate_branch(&self, state: MinesweeperState, mut group: Group) -> Option<Evaluation> {
        let idx = group.pop()?; // TODO
        log::debug!("{}", idx);
        let (s0, s1) = group.split(idx);
        let mut sf = state.clone();
        let mut sm = state.clone(); // move
                                    // Short circuiting
        let sf = (sf.set_flag(idx) && self.make_consistent_sq(idx, &mut sf)).then(|| sf);
        let sm = (sm.set_mark(idx) && self.make_consistent_sq(idx, &mut sm)).then(|| sm);
        match (sf, sm) {
            (Some(sf), Some(sm)) => {
                match (
                    self.evaluate_splitted(idx, sf, s0.clone(), s1.clone()),
                    self.evaluate_splitted(idx, sm, s0, s1),
                ) {
                    (Some(e0), Some(e1)) => Some(e0 + e1),
                    (e0, e1) => e0.or(e1),
                }
            }
            (sf, sm) => sf
                .or(sm)
                .and_then(|st| self.evaluate_splitted(idx, st, s0, s1)),
        }
    }

    fn evaluate_splitted(
        &self,
        idx: Index,
        state: MinesweeperState,
        s0: Option<Group>,
        s1: Option<Group>,
    ) -> Option<Evaluation> {
        log::debug!("{}", ShowState { bombs: None, config: &self.config, state: &state });
        log::debug!("{:?}", s0);
        log::debug!("{:?}", s1);
        let es = Evaluation::new(&state, idx);
        match (
            s0.and_then(|s| s.trimmed(&state)),
            s1.and_then(|s| s.trimmed(&state)),
        ) {
            (Some(s0), Some(s1)) => {
                let e0 = self.evaluate_branch(state.clone(), s0)?;
                let e1 = self.evaluate_branch(state.clone(), s1)?;
                Some(es * e0 * e1)
            }
            (s0, s1) => {
                log::debug!("{:?}", s0);
                log::debug!("{:?}", s1);
                if let Some(s) = s0.or(s1) {
                    Some(es * self.evaluate_branch(state, s)?)
                } else {
                    Some(es)
                }
            }
        }
    }

    fn center_search(&self, state: &MinesweeperState) -> Option<ScoredIndex> {
        let center = self.config.center();
        match state.get(center) {
            Status::Known(_) => None,
            Status::Flagged | Status::Marked => unreachable!(),
            _ => Some((R64::new(0.0), center)),
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
        self.center_search(state)
            .or_else(|| Self::fast_search(state))
            .or_else(|| {
                self.make_consistent_all(state).then(|| ())?;
                Self::fast_search(state).or_else(|| {
                    let group = Group::new(&self, state)?;
                    let unknowns_count = group.count_unknowns();
                    log::debug!("{:?}", group);
                    let eval = self.evaluate_branch(state.clone(), group)?;
                    log::info!("{:?}", eval);
                    eval.label_certains(state);
                    Self::fast_search(state)
                })
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

/*
fn lift<T, F: Fn(T, T) -> T>(f: F) -> impl Fn(Option<T>, Option<T>) -> Option<T> {
    move |a, b| match (a, b) {
        (Some(x), Some(y)) => Some(f(x, y)),
        (a, b) => a.or(b),
    }
}
*/