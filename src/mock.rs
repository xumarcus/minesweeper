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

use rand::{
    self,
    distributions::{Distribution, Uniform},
};

pub struct MockMinesweeper {
    state: MinesweeperState,
    bombs: Vec<bool>,
}

impl MockMinesweeper {
    pub fn new(width: usize, length: usize, mines: usize) -> MsResult<Self> {
        let state = MinesweeperState::new(width, length, mines)?;
        let mut rng = rand::thread_rng();
        let w_gen = Uniform::from(0..width);
        let l_gen = Uniform::from(0..length);
        let mut bombs = vec![false; state.size()];
        for _ in 0..mines {
            loop {
                let row = w_gen.sample(&mut rng);
                let col = l_gen.sample(&mut rng);
                let idx = state.from_rc(row, col);
                if idx != state.center() && !bombs[idx] {
                    bombs[idx] = true;
                    break;
                }
            }
        }
        Ok(Self { state, bombs })
    }

    #[rustfmt::skip]
    pub fn from_difficulty(diff: Difficulty) -> Self {
        let result = match diff {
            Difficulty::Beginner     => Self::new( 9,  9, 10),
            Difficulty::Intermediate => Self::new(16, 16, 40),
            Difficulty::Expert       => Self::new(30, 16, 99),
        };
        result.unwrap()
    }

    fn set_known(&mut self, idx: usize) -> MsResult<()> {
        let Self { state, bombs } = self;
        match state.board[idx] {
            Status::Flagged => return Err(MinesweeperError::FlaggedButNotBomb),
            Status::Known(_) => (),
            _ => {
                let count = state.square(idx).filter(|cidx| bombs[*cidx]).count();
                state.board[idx] = Status::Known(count);
                if count == 0 { // Should have no flags as count is zero
                    for cidx in state.square(idx) {
                        self.set_known(cidx)?;
                    }
                }
            }
        }
        Ok(())
    }
}

impl Minesweeper for MockMinesweeper {
    fn get(&self) -> &MinesweeperState {
        &self.state
    }

    fn get_bombs(&self) -> Option<&Vec<bool>> {
        Some(&self.bombs)
    }

    fn set(&mut self, state: MinesweeperState) {
        self.state = state;
    }

    fn reveal(&mut self, idx: usize) -> MsResult<()> {
        if self.bombs[idx] {
            return Err(MinesweeperError::RevealedBomb);
        }
        self.set_known(idx)
    }
}