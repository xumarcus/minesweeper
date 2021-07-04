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

pub struct MockMinesweeper {
    bombs: Vec<bool>,
    config: Config,
    state: MinesweeperState,
}

impl MockMinesweeper {
    pub fn new(config: Config) -> Self {
        let state = MinesweeperState::new(&config);
        let mut rng = config.new_rng();
        let mut bombs = vec![false; config.size()];
        for _ in 0..config.mines() {
            loop {
                let idx = config.random_index(&mut rng);
                if idx != config.center() && !bombs[idx] {
                    bombs[idx] = true;
                    break;
                }
            }
        }
        Self {
            bombs,
            config,
            state,
        }
    }
}

impl Minesweeper for MockMinesweeper {
    fn get_bombs(&self) -> Option<&[bool]> {
        Some(&self.bombs)
    }

    fn get_config(&self) -> &Config {
        &self.config
    }

    fn get_state(&self) -> &MinesweeperState {
        &self.state
    }

    fn pull(&self) -> MsResult<MinesweeperState> {
        Ok(self.state.clone())
    }

    fn flag(&mut self, idx: usize) -> MsResult<()> {
        debug_assert!(self.bombs[idx]);
        debug_assert!(self.state.board()[idx] != Status::Flagged);
        Ok(()) // Noop cuz mock
    }

    fn reveal(&mut self, idx: usize) -> MsResult<()> {
        (!self.bombs[idx])
            .then(|| self.state.reveal(idx, &self.bombs, self.config))
            .ok_or(MinesweeperError::RevealedBomb(idx))
    }

    fn set_internal(&mut self, state: MinesweeperState) -> MsResult<()> {
        self.state = state;
        Ok(())
    }
}
