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

use std::cmp::{min, max};
use std::fmt::{self, Display};

struct Group {
    knowns: BitVec,
     unknowns: BitVec,
}

pub struct Solver<T: Sized + Minesweeper> {
    config: Config,
    squares: Vec<Square>,
    sweep: T,
}

impl<T: Sized + Minesweeper> Solver<T> {
    pub fn new(config: Config, sweep: T) -> Self {
        let squares = (0..config.size())
            .map(|idx| {
                let (row, col) = config.as_rc(idx);
                let rmin = max(1, row) - 1;
                let rmax = min(config.width() - 1, row + 1);
                let cmin = max(1, col) - 1;
                let cmax = min(config.length() - 1, col + 1);
                (rmin..=rmax)
                    .flat_map(|r| (cmin..=cmax).map(move |c| config.from_rc(r, c)))
                    .filter(|cidx| cidx != &idx)
                    .collect()
            })
            .collect();
        Self { config, sweep, squares }
    }

    pub fn solve(&mut self) -> MsResult<()> {
        while let Some(x) = self.next() {
            log::trace!("{}", self);
            drop(x?);
        }
        assert_eq!(self.sweep.get_state().unknowns(), 0);
        Ok(())
    }

    pub fn solve_next(&mut self) -> MsResult<Option<ScoredIndex>> {
        let mut state = self.sweep.pull()?;
        let scored_index = self.search(&mut state);
        self.sweep.push(state)?;
        if let Some((_, idx)) = scored_index {
            match self.sweep.reveal(idx) {
                Ok(()) => Ok(scored_index),
                Err(e) => {
                    log::info!("Error {:?}", e);
                    Err(e)
                }
            }
        } else {
            Ok(None)
        }
    }

    fn square_of<'a, F: 'static + Fn(Status) -> bool>(&'a self, state: &'a MinesweeperState, idx: Index, f: F) -> impl Iterator<Item = Index> + 'a {
        self.squares[idx].iter()
            .filter_map(move |&cidx| f(state.get(cidx)).then(|| cidx))
    }

    fn make_consistent(&self, state: &mut MinesweeperState, idx: Index) -> MsResult<()> {
        if let Some(count) = state.get_known(idx) {
            let minimum = self.square_of(state, idx, |status| status == Status::Flagged).count();
            let maximum = self.square_of(state, idx, |status| status == Status::Unknown).count() + minimum;
            if !(minimum..=maximum).contains(&count) {
                return Err(MinesweeperError::NumberOfMinesOutOfRange);
            }
            for cidx in self.square_of(state, idx, |status| status == Status::Unknown) {
                if count == minimum && state.get(cidx) != Status::Marked {
                    state.set_mark(cidx);
                } else if count == maximum && state.get(cidx) != Status::Flagged {
                    state.set_flag(cidx);
                } else {
                    continue;
                }
                self.make_consistent_sq(state, cidx)?;
            }
        }
        Ok(())
    }

    fn make_consistent_sq(&self, state: &mut MinesweeperState, idx: Index) -> MsResult<()> {
        for cidx in self.squares[idx] {
            self.make_consistent(state, cidx)?;
        }
        Ok(())
    }

    fn make_consistent_all(&self, state: &mut MinesweeperState) -> MsResult<()> {
        for idx in 0..self.config.size() {
            self.make_consistent(state, idx)?;
        }
        Ok(())
    }

    fn search(&self, state: &mut MinesweeperState) -> Option<ScoredIndex> {
        self.center_search(state)
            .or_else(|| self.fast_search(state))
            .or_else(|| {
                self.make_consistent_all(state).unwrap();
                self.fast_search(state)
            })
            .or_else(|| {

                self.fast_search(state)
                .or_else(|| self.probabilistic_search(state))
            })
    }

    fn split_group(&self, idx: Index, mut group: Group) -> (Group, Group) {
        let mut stack = bitvec![0; self.config.size()];
        stack.set(idx, true);
        let mut other = Group {
            knowns: bitvec![0; self.config.size()],
            unknowns: bitvec![0; self.config.size()],
        };
        while let Some(cur) = stack.first_one() {
            stack.set(cur, false);
            if group.knowns[cur] {
                for cidx in self.squares[cur] {
                    stack.set(cidx, true);
                }
                group.knowns.set(cur, false);
                other.knowns.set(cur, true);
            }
            if group.unknowns[cur] {
                for cidx in self.squares[cur].iter().filter(|&&cidx| group.knowns[cidx]) {
                    stack.set(*cidx, true);
                }
                group.unknowns.set(cur, false);
                other.unknowns.set(cur, true);
            }
        }
        (group, other)
    }

    fn evaluate(&self, state: MinesweeperState, group: Group) -> Option<Evaluation> {
        let idx = group.unknowns.first_one()?;  // TODO
        let (subset_0, subset_1) = self.split_group(idx, group);
        if group.is_empty() {
            return None;
        }
        let recur = |state_: MinesweeperState| {
            self.make_consistent(&mut state_, idx)
                .ok()
                .and_then(|_| {
                    let unknowns = state_.unknowns();
                    let flags_remaining = state_.flags_remaining();
                    Evaluation::add(Evaluation::new(&state_, idx)
                    let e_0 = self.evaluate(state_.clone(), subset_0)?;
                    let e_1 = self.evaluate(state_, subset_1)?;
                    let e_3 = util::lift(e_0, e_1, |a, b| Evaluation::add_disj_group_evals(u, f, a, b));
                })
        };
        let mut state_to_flag = state; // move
        let mut state_to_mark = state.clone();
        state_to_flag.set_flag(idx);
        state_to_mark.set_mark(idx);
        util::lift(recur(state_to_flag), recur(state_to_mark), Evaluation::add_same_group_evals)
    }

    fn center_search(&self, state: &MinesweeperState) -> Option<ScoredIndex> {
        let center = self.config.center();
        match state.get(center) {
            Status::Known(_) => None,
            Status::Flagged | Status::Marked => unreachable!(),
            _ => Some((R64::new(1.0), center)),
        }
    }

    fn fast_search(&self, state: &MinesweeperState) -> Option<ScoredIndex> {
        let idx = state
            .board()
            .iter()
            .position(|status| status == &Status::Marked)?;
        Some((R64::new(1.0), idx))
    }

    fn probabilistic_search(
        &self,
        state: &MinesweeperState,
        group: &[Index],
        eval: &Evaluation,
    ) -> Option<(R64, Index)> {
        let Evaluation {
            conf_counts,
            mark_counts,
        } = eval;
        let unknowns_otherwise = group
            .iter()
            .filter(|idx| state.get(**idx) == Status::Unknown)
            .count();
        let base = R64::try_new((state.flags_remaining() as f64) / (state.unknowns() as f64))?.raw();
        debug_assert!(0.0 <= base && base <= 1.0);
        mark_counts
            .iter()
            .enumerate()
            .map(|(group_idx, mark_count)| {
                let idx = group[group_idx];
                let mark_prob = mark_count
                    .iter()
                    .zip(conf_counts.iter())
                    .enumerate()
                    .take(unknowns_otherwise + 1)
                    .filter_map(|(i, (mark, conf))| {
                        if state.get(idx) != Status::Unknown {
                            return None;
                        }
                        let p = (*mark as f64) / (*conf as f64)
                            * Binomial::new(unknowns_otherwise, base).mass(i);
                        R64::try_new(p)
                    })
                    .sum::<R64>();
                (mark_prob, idx)
            })
            .max()
    }
}

impl<T: Sized + Minesweeper> Iterator for Solver<T> {
    type Item = MsResult<ScoredIndex>;
    fn next(&mut self) -> Option<Self::Item> {
        self.solve_next().transpose()
    }
}

impl<T: Sized + Minesweeper> Display for Solver<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let bombs = self.sweep.get_bombs();
        let state = self.sweep.get_state();
        writeln!(f, "{:?}", self.config)?;
        write!(f, "{}", ShowState::new(state, bombs))
    }
}
