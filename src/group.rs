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

#[derive(Clone)]
pub struct Group<'a> {
    knowns: BitVec,
    solver: &'a Solver,
    unknowns: BitVec,
}

impl<'a> IntoIterator for Group<'a> {
    type Item = Self;
    type IntoIter = IntoIter<'a>;
    fn into_iter(self) -> Self::IntoIter {
        IntoIter(self)
    }
}

impl<'a> fmt::Debug for Group<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let knowns = self.knowns.iter_ones().collect::<Vec<Index>>();
        let unknowns = self.unknowns.iter_ones().collect::<Vec<Index>>();
        writeln!(f, "Group (K{:?}, U{:?})", knowns, unknowns)
    }
}

impl<'a> Group<'a> {
    fn as_option(self) -> Option<Self> {
        (self.knowns.any() && self.unknowns.any()).then(|| self)
    }

    fn zero(&self) -> BitVec {
        bitvec![0; self.solver.size()]
    }

    fn from(solver: &'a Solver) -> Self {
        Group {
            knowns: bitvec![0; solver.size()],
            solver,
            unknowns: bitvec![0; solver.size()],
        }
    }

    pub fn new(solver: &'a Solver, state: &MinesweeperState) -> (Option<Self>, BitVec) {
        let mut group = Group::from(solver);
        for (idx, status) in state.board().iter().enumerate() {
            match status {
                Status::Flagged | Status::Marked | Status::Known(0) => continue,
                Status::Known(_) => group.knowns.set(idx, true),
                Status::Unknown => group.unknowns.set(idx, true),
            }
        }
        group.trim(state)
    }

    pub fn trim(&self, state: &MinesweeperState) -> (Option<Self>, BitVec) {
        let mut group = self.clone();
        let mut remainder = bitvec![0; self.solver.size()];
        loop {
            let mut knowns = self.zero();
            let mut unknowns = self.zero();
            for idx in group.unknowns.iter_ones() {
                if matches!(state.get(idx), Status::Unknown)
                    && self
                        .solver
                        .square(idx)
                        .iter()
                        .any(|&cidx| matches!(state.get(cidx), Status::Known(_)))
                {
                    unknowns.set(idx, true);
                } else {
                    remainder.set(idx, true);
                }
            }
            for idx in group.knowns.iter_ones() {
                if self
                    .solver
                    .square(idx)
                    .iter()
                    .any(|&cidx| matches!(state.get(cidx), Status::Unknown))
                {
                    knowns.set(idx, true);
                }
            }
            if knowns == group.knowns && unknowns == group.unknowns {
                break (group.as_option(), remainder);
            } else {
                group.knowns = knowns;
                group.unknowns = unknowns;
            }
        }
    }

    pub fn get(&self) -> Option<Index> {
        self.unknowns.iter_ones().max_by_key(|&idx| {
            self.solver
                .square(idx)
                .iter()
                .filter(|&&cidx| self.knowns[cidx])
                .count()
        })
    }
}

pub struct IntoIter<'a>(Group<'a>);

impl<'a> Iterator for IntoIter<'a> {
    type Item = Group<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        let idx = self.0.unknowns.first_one()?;
        let mut split = Group::from(self.0.solver);
        let mut stack = self.0.zero();
        stack.set(idx, true);
        while let Some(cur) = stack.first_one() {
            stack.set(cur, false);
            debug_assert_ne!(self.0.knowns[cur], self.0.unknowns[cur]);
            if self.0.knowns[cur] {
                for cidx in self.0.solver.square(cur) {
                    if self.0.unknowns[*cidx] {
                        stack.set(*cidx, true);
                    }
                }
                self.0.knowns.set(cur, false);
                split.knowns.set(cur, true);
            }
            if self.0.unknowns[cur] {
                for cidx in self.0.solver.square(cur) {
                    if self.0.knowns[*cidx] {
                        stack.set(*cidx, true);
                    }
                }
                self.0.unknowns.set(cur, false);
                split.unknowns.set(cur, true);
            }
        }
        Some(split)
    }
}
