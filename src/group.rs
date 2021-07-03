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
pub struct Group<'a> {
    knowns: BitVec,
    solver: &'a Solver,
    unknowns: BitVec,
}

impl<'a> Group<'a> {
    fn zero(solver: &'a Solver) -> Self {
        let knowns = bitvec![0; solver.size()];
        let unknowns = bitvec![0; solver.size()];
        Self { knowns, solver, unknowns }
    }

    fn as_option(self) -> Option<Self> {
        (self.knowns.any() && self.unknowns.any()).then(|| self)
    }

    pub fn new(solver: &'a Solver, state: &MinesweeperState) -> Option<Self> {
        let mut group = Group::zero(solver);
        for (idx, status) in state.board().iter().enumerate() {
            match status {
                Status::Flagged | Status::Marked | Status::Known(0) => continue,
                Status::Known(_) => {
                    let has_unknown = solver.square(idx)
                        .iter()
                        .any(|&cidx| state.get(cidx) == Status::Unknown);
                    if has_unknown {
                        group.knowns.set(idx, true);
                    }
                }
                Status::Unknown => {
                    let has_known = solver.square(idx)
                        .iter()
                        .any(|&cidx| matches!(state.get(cidx), Status::Known(_)));
                    if has_known {
                        group.unknowns.set(idx, true);
                    }
                }
            }
        }
        group.as_option()
    }

    pub fn count_unknowns(&self) -> usize {
        self.unknowns.count_ones()
    }

    pub fn get(&self, state: &MinesweeperState) -> Option<Index> {
        unimplemented!()
    }

    pub fn split(mut self, idx: Index) -> (Self, Option<Self>) {
        self.unknowns.set(idx, false);
        let mut stack = bitvec![0; self.solver.size()];
        let mut other = Group::zero(self.solver);
        if let Some(&sidx) = self.solver.square(idx).iter().filter(|&&cidx| self.knowns[cidx]).next() {
            stack.set(sidx, true);
        } else {
            return (self, None);
        }
        while let Some(cur) = stack.first_one() {
            stack.set(cur, false);
            debug_assert_ne!(self.knowns[cur], self.unknowns[cur]);
            if self.knowns[cur] {
                for cidx in self.solver.square(cur) {
                    if self.unknowns[*cidx] {
                        stack.set(*cidx, true);
                    }
                }
                self.unknowns.set(cur, false);
                other.unknowns.set(cur, true);
            }
            if self.unknowns[cur] {
                for cidx in self.solver.square(cur) {
                    if self.knowns[*cidx] {
                        stack.set(*cidx, true);
                    }
                }
                self.knowns.set(cur, false);
                other.knowns.set(cur, true);
            }
        }
        let s = other.as_option();
        match self.as_option() {
            Some(x) => (x, s),
            x => (s.unwrap(), x)
        }
    }
}