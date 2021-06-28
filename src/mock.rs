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
    rngs::StdRng,
    SeedableRng,
};

pub struct MockMinesweeper {
    state: MinesweeperState,
    bombs: Vec<bool>,
}

impl MockMinesweeper {
    pub fn new(width: usize, length: usize, mines: usize, seed: Option<u64>) -> MsResult<Self> {
        let state = MinesweeperState::new(width, length, mines)?;
        let seed = seed.unwrap_or_else(|| rand::random::<u64>());
        log::debug!("Create mock with seed {}", seed);
        let mut rng = StdRng::seed_from_u64(seed);
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
    pub fn from_difficulty(diff: Difficulty, seed: Option<u64>) -> Self {
        let result = match diff {
            Difficulty::Beginner     => Self::new( 9,  9, 10, seed),
            Difficulty::Intermediate => Self::new(16, 16, 40, seed),
            Difficulty::Expert       => Self::new(16, 30, 99, seed),
        };
        result.unwrap()
    }
}

impl Minesweeper for MockMinesweeper {
    fn get_bombs(&self) -> Option<&Vec<bool>> {
        Some(&self.bombs)
    }

    fn get_state(&self) -> &MinesweeperState {
        &self.state
    }

    fn pull(&mut self) -> MsResult<MinesweeperState> {
        Ok(self.state.clone())
    }

    fn flag(&mut self, idx: usize) -> MsResult<()> {
        debug_assert!(self.bombs[idx] && self.state.board()[idx] != Status::Flagged);
        Ok(()) // Noop cuz mock
    }

    fn reveal(&mut self, idx: usize) -> MsResult<()> {
        (!self.bombs[idx])
        .then(|| self.state.set_known(idx, &self.bombs))
        .ok_or(MinesweeperError::RevealedBomb(idx))
    }

    fn set_internal(&mut self, state: MinesweeperState) -> MsResult<()> {
        self.state = state;
        Ok(())
    }
}
