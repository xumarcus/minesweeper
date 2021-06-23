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

pub struct Show<'a, T: Minesweeper>(pub &'a T);

impl<'a, T: Minesweeper> fmt::Display for Show<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let state = self.0.get();
        let flagged_count = state.board
            .iter()
            .filter(|status| status == &&Status::Flagged)
            .count();
        writeln!(f, "Dimensions: {} x {}", state.width(), state.length())?;
        writeln!(f, "Flagged: {} / {}", flagged_count, state.mines())?;
        for row in 0..state.width() {
            let t = row * state.length();
            for idx in t..t + state.length() {
                if let Some(bombs) = self.0.get_bombs() {
                    if bombs[idx] {
                        match state.board[idx] {
                            Status::Flagged => write!(f, "🚩")?,
                            Status::Known(_) => unreachable!("Is bomb"),
                            Status::Marked => unreachable!("Wrong solution"),
                            Status::Unknown => write!(f, "💣")?,
                        }
                        continue;
                    }
                }
                match state.board[idx] {
                    Status::Flagged => unreachable!("Wrong flag"),
                    Status::Known(x) => write!(f, "{}.", x)?,
                    Status::Marked => write!(f, "✔️")?,
                    Status::Unknown => write!(f, "❔")?,
                }
            }
            write!(f, "\n")?;
        }
        Ok(())
    }
}