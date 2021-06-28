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
        let length = self.0.get_state().length();
        for x in self {
            let (p, idx) = x?;
            let row = idx / length;
            let col = idx % length;
            log::info!("Guess ({:02}, {:02}): {:.1}%", row, col, p * 100.0);
        }
        Ok(())
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
