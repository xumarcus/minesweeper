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

pub struct ShowState<'a> {
    bombs: Option<&'a [bool]>,
    config: &'a Config,
    state: &'a MinesweeperState,
}

impl <'a> ShowState<'a> {
    #[allow(dead_code)]
    pub fn from_state(config: &'a Config, state: &'a MinesweeperState) -> Self {
        Self { bombs: None, config, state }
    }
}

impl<'a> fmt::Display for ShowState<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let flags = self.state.flags_remaining();
        writeln!(f, "[Remain {:02} flags] {:?}", flags, self.config)?;
        for (idx, status) in self.state.board().iter().enumerate() {
            if idx % self.config.length() == 0 {
                write!(f, "\n")?;
            }
            if self
                .bombs
                .and_then(|bombs| bombs.get(idx))
                .cloned()
                .unwrap_or(false)
            {
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

pub struct ShowMinesweeper<'a, T: Minesweeper>(pub &'a T);

impl<'a, T: Minesweeper> fmt::Display for ShowMinesweeper<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let show_state = ShowState {
            bombs: self.0.get_bombs(),
            config: self.0.get_config(),
            state: self.0.get_state(),
        };
        write!(f, "{}", show_state)
    }
}
