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
    pub(super) board: Vec<MockCell>,
    pub(super) config: Config,
}

impl MockMinesweeper {
    pub fn new(config: Config) -> Self {
        let mut inst = Self { board: Vec::new(), config };
        let mut rng = rand::thread_rng();
        let w_gen = Uniform::from(0..config.width);
        let l_gen = Uniform::from(0..config.length);
        let mut is_bombs = vec![false; config.size()];
        for _ in 0..config.mines {
            loop {
                let row = w_gen.sample(&mut rng);
                let col = l_gen.sample(&mut rng);
                let idx = config.from_rc(row, col);
                if idx != config.from_rc(config.width / 2, config.length / 2) && !is_bombs[idx] {
                    is_bombs[idx] = true;
                    break;
                }
            }
        }
        inst.board = is_bombs.into_iter().map(MockCell::new).collect();
        inst
    }
}

impl Minesweeper for MockMinesweeper {
    type Item = MockCell;

    fn reveal(&mut self, idx: usize) -> MsResult<()> {
        if self.board[idx].is_bomb {
            
        }

    }

    fn get_config(&self) -> Config {
        self.config
    }

    fn get_cells(&self) -> &Vec<Self::Item> {
        &self.board
    }

    fn get_cells_mut(&mut