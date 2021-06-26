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
        while let Some((idx, p)) = self.0.step()? {
            log::debug!("Guess {:?}: {:.1}%", self.0.get_state().as_rc(idx), p * 100.0);
            log::trace!("{}", self);
        }
        Ok(())
    }
}

impl<T: Sized + Minesweeper> fmt::Display for Solver<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let state = self.0.get_state();
        writeln!(f, "Dimensions: {} x {}", state.width(), state.length())?;
        writeln!(f, "Flagged: {} / {}", state.count(Status::Flagged), state.mines())?;
        for (idx, status) in state.board().iter().enumerate() {
            if idx % state.length() == 0 {
                write!(f, "\n")?;
            }
            if *self.0.get_bombs().and_then(|bombs| bombs.get(idx)).unwrap_or(&false) {
                match status {
                    Status::Flagged => write!(f, "🚩")?,
                    Status::Known(_) => unreachable!("Is bomb"),
                    Status::Marked => unreachable!("Wrong solution"),
                    Status::Unknown => write!(f, "💣")?,
                }
            } else {
                match status {
                    Status::Flagged => unreachable!("Wrong flag"),
                    Status::Known(x) => write!(f, "{}.", x)?,
                    Status::Marked => write!(f, "✔️")?,
                    Status::Unknown => write!(f, "❔")?,
                }
            }
        }
        Ok(())
    }
}