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

impl<T: Sized + Minesweeper> Display for Show<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let bombs = self.sweep.get_bombs();
        let state = self.sweep.get_state();
        writeln!(f, "{:?}; {}", self.config, state.flags_remaining())?;
        for (idx, status) in state.board().iter().enumerate() {
            if idx % self.config.length() == 0 {
                write!(f, "\n")?;
            }
            if bombs.and_then(|bombs| bombs.get(idx)).cloned().unwrap_or(false) {
                match status {
                    Status::Flagged => write!(f, "🚩")?,
                    Status::Known(_) => unreachable!("Is bomb"),
                    Status::Marked => unreachable!("Wrong solution"),
                    Status::Unknown => write!(f, "💣")?,
                }
            } else {
                match status {
                    Status::Flagged => write!(f, "🏁")?,
                    Status::Known(x) => write!(f, "{}.", x)?,
                    Status::Marked => write!(f, "✅")?,
                    Status::Unknown => write!(f, "❔")?,
                }
            }
        }
        Ok(())
    }
}