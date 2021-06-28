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

use std::fmt::{self, Display};

pub struct Solver<T: Sized + Minesweeper>(T);

impl<T: Sized + Minesweeper> Solver<T> {
    pub fn new(inst: T) -> Self {
        Self(inst)
    }

    pub fn solve(&mut self) -> MsResult<()> {
        while let Some(x) = self.next() {
            log::trace!("{}", self);
            drop(x?);
        }
        let unknowns = self.0.get_state().unknowns();
        (unknowns == 0)
            .then(|| ())
            .ok_or(MinesweeperError::SolverEarlyExit(unknowns))
    }
}

impl<T: Sized + Minesweeper> Iterator for Solver<T> {
    type Item = MsResult<ScoredUnknown>;
    fn next(&mut self) -> Option<Self::Item> {
        self.0.step().transpose()
    }
}

impl<T: Sized + Minesweeper> Display for Solver<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let state = self.0.get_state();
        let bombs = self.0.get_bombs();
        write!(f, "{}", ShowState::new(state, bombs))
    }
}
