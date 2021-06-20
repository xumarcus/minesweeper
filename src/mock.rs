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

use rand::{self, distributions::{Distribution, Uniform}};

pub struct MockMinesweeper {
    pub(super) board: Vec<Status>,
    pub(super) config: Config,
    is_bomb: Vec<bool>,
}

impl MockMinesweeper {
    pub fn new(config: Config) -> Self {
        let mut rng = rand::thread_rng();
        let w_gen = Uniform::from(0..config.width);
        let l_gen = Uniform::from(0..config.length);
        let mut is_bomb = vec![false; config.size()];
        for _ in 0..config.mines {
            loop {
                let row = w_gen.sample(&mut rng);
                let col = l_gen.sample(&mut rng);
                let idx = config.from_rc(row, col);
                if idx != config.from_rc(config.width / 2, config.length / 2) && !is_bomb[idx] {
                    is_bomb[idx] = true;
                    break;
                }
            }
        }
        let board = vec![Status::default(); config.size()];
        Self { board, config, is_bomb }
    }
}

impl fmt::Display for MockMinesweeper {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { board, config, is_bomb } = self;
        let flagged_count = board.iter().filter(|status| status == &&Status::Flagged).count();
        writeln!(f, "Dimensions: {} x {}", config.width, config.length)?;
        writeln!(f, "Flagged: {} / {}", flagged_count, config.mines)?;
        for row in 0..config.width {
            let sidx = row * config.length;
            for idx in sidx .. sidx + config.length {
                if is_bomb[idx] {
                    match board[idx] {
                        Status::Flagged => write!(f, "🚩")?,
                        Status::Marked => unreachable!("Wrong solution"),
                        Status::Unknown => write!(f, "💣")?,
                        Status::Known(_) => unreachable!("Is bomb"),
                    }
                } else {
                    match board[idx] {
                        Status::Flagged => unreachable!("Wrong flag"),
                        Status::Marked => write!(f, "✔️")?,
                        Status::Unknown => write!(f, "❔")?,
                        Status::Known(x) => write!(f, "{}.", x)?,
                    }
                }
            }
            write!(f, "\n")?;
        }
        Ok(())
    }
}

impl Minesweeper for MockMinesweeper {
    fn reveal(&mut self, idx: usize) -> MsResult<()> {
        if self.is_bomb[idx] {
            Err(MinesweeperError::RevealedBomb)
        } else {
            Ok(())
        }
    }

    fn get_config(&self) -> Config {
        self.config
    }

    fn get_cells(&self) -> &Vec<Status> {
        &self.board
    }

    fn get_cells_mut(&mut self) -> &mut Vec<Status> {
        &mut self.board
    }
}