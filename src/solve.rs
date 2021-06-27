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

pub struct Solver<T: Sized + Minesweeper>(T);

impl<T: Sized + Minesweeper> Solver<T> {
    pub fn new(inst: T) -> Self {
        Self(inst)
    }

    pub fn solve(&mut self) -> MsResult<()> {
        log::trace!("{}", self);
        while let Some((p, idx)) = self.0.step()? {
            let percent = (*p.numer() as f64) / (*p.denom() as f64) * 100.0;
            let (row, col) = self.0.get_state().as_rc(idx);
            log::debug!("Guess ({:02}, {:02}): {:.1}%", row, col, percent);
            log::trace!("{}", self);
        }
        Ok(())
    }
}

impl<T: Sized + Minesweeper> fmt::Display for Solver<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let state = self.0.get_state();
        let bombs = self.0.get_bombs();
        write!(f, "{}", ShowState::new(state, bombs))
    }
}
