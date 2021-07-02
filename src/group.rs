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
pub struct Group {
    knowns: BitVec,
    unknowns: BitVec,
}

impl Group {
    pub fn new(state: &MinesweeperState) -> Self {
        let mut knowns = bitvec![0; state.size()];
        let mut unknowns = bitvec![0; state.size()];
        for (idx, status) in state.board().iter().enumerate() {
            match status {
                Status::Flagged | Status::Marked => continue,
                Status::Known(_) => knowns.set(idx, true),
                Status::Unknown => unknowns.set(idx, true)
            }
        }
        Self { knowns, unknowns }
    }

    pub fn get(&self, state: &MinesweeperState) -> Option<Index> {
        unimplemented!()
    }

    pub fn knowns(&self) -> &BitVec {
        &self.knowns
    }

    pub fn unknowns(&self) -> &BitVec {
        &self.unknowns
    }

    pub fn is_empty(&self) -> bool {
        self.unknowns.not_any()
    }

    pub fn split(mut self, idx: Index, config: &Config) -> (Group, Group) {
        let mut stack = bitvec![0; config.size()];
        stack.set(idx, true);
        let mut other = Group {
            knowns: bitvec![0; config.size()],
            unknowns: bitvec![0; config.size()],
        };
        while let Some(cur) = stack.first_one() {
            stack.set(cur, false);
            if self.unknowns[cur] {
                for cidx in config.square(cur) {
                    if self.knowns[*cidx] {
                        stack.set(*cidx, true);
                    }
                }
                self.unknowns.set(cur, false);
                other.unknowns.set(cur, true);
            } else {
                for cidx in config.square(cur) {
                    if self.knowns[*cidx] || self.unknowns[*cidx] {
                        stack.set(*cidx, true);
                    }
                }
                if self.knowns[cur] {
                    self.knowns.set(cur, false);
                    other.knowns.set(cur, true);
                } else {
                    log::debug!("Group::split {}", cur);
                }
            }
        }
        if self.is_empty() {
            (other, self)
        } else {
            (self, other)
        }
    }
}