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

use std::ops::{Add, Mul};

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
                .collect()
        }
    }

    pub fn size(&self) -> usize {
        self.squares.len()
    }

    pub fn square(&self, idx: Index) -> &[Index] {
        &self.squares[idx]
    }

    fn square_of<'a, F: 'static + Fn(Status) -> bool>(&'a self, state: &'a MinesweeperState, idx: Index, f: F) -> impl Iterator<Item = Index> + 'a {
        self.square(idx).iter()
            .filter_map(move |&cidx| f(state.get(cidx)).then(|| cidx))
    }

    fn make_consistent(&self, idx: Index, state: &mut MinesweeperState) -> bool {
        state.get_known(idx)
            .map(|count| {
                let minimum = self.square_of(state, idx, |status| status == Status::Flagged).count();
                let maximum = self.square_of(state, idx, |status| status == Status::Unknown).count() + minimum;
                if !(minimum..=maximum).contains(&count) {
                    return false;
                }
                let square = self.square_of(state, idx, |status| status == Status::Unknown).collect::<Square>();
                square.into_iter().all(|cidx| {
                    if count == minimum && state.get(cidx) != Status::Marked {
                        state.set_mark(cidx);
                    } else if count == maximum && state.get(cidx) != Status::Flagged {
                        state.set_flag(cidx);
                    } else {
                        return true;
                    }
                    self.make_consistent_sq(cidx, state)
                })
            })
            .unwrap_or(true)
    }

    fn make_consistent_sq(&self, idx: Index, state: &mut MinesweeperState) -> bool {
        self.square(idx).iter().all(|&cidx| self.make_consistent(cidx, state))
    }
    
    fn make_consistent_all(&self, state: &mut MinesweeperState) -> bool {
        (0..self.size()).all(|cidx| self.make_consistent(cidx, state))
    }

    fn evaluate_branch(&self, state: MinesweeperState, group: Group) -> Option<Evaluation> {
        let idx = group.get(&state)?;  // TODO
        let (group, splitted) = group.split(idx);
        let es = Evaluation::new(&state, idx);
        let mut sf = state.clone();
        let mut sm = state; // move
        sf.set_flag(idx);
        sm.set_mark(idx);
        let sf = self.make_consistent(idx, &mut sf).then(|| sf);
        let sm = self.make_consistent(idx, &mut sm).then(|| sm);
        let el = match (sf, sm) {
            (Some(sf), Some(sm)) => lift(Evaluation::add)(
                self.evaluate_splitted(sf, group.clone(), splitted.clone()),
                self.evaluate_splitted(sm, group, splitted)
            ),
            (sf, sm) => sf.or(sm).and_then(|st| self.evaluate_splitted(st, group, splitted))
        };
        Some(es * el?)
    }

    fn evaluate_splitted(&self, state: MinesweeperState, group: Group, splitted: Option<Group>) -> Option<Evaluation> {
        if let Some(splitted) = splitted {
            let es = self.evaluate_branch(state.clone(), splitted)?;
            let eg = self.evaluate_branch(state, group)?;
            Some(es * eg)
        } else {
            self.evaluate_branch(state, group)
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

    pub fn solve_next(&self, state: &mut MinesweeperState) -> Option<ScoredIndex> {
        self.center_search(state)
            .or_else(|| Self::fast_search(state))
            .or_else(|| {
                assert!(self.make_consistent_all(state));
                Self::fast_search(state)
            })
            .or_else(|| {
                let group = Group::new(&self, state)?;
                let eval = self.evaluate_branch(state.clone(), group)?;
                log::debug!("{:?}", eval);
                Self::fast_search(state)
            })
    }
}

fn lift<T, F: Fn(T, T) -> T>(f: F) -> impl Fn(Option<T>, Option<T>) -> Option<T> {
    move |a, b| match (a, b) {
        (Some(x), Some(y)) => Some(f(x, y)),
        (a, b) => a.or(b),
    }
}
