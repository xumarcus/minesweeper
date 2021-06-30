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
    state: MinesweeperState,
    bombs: Vec<bool>,
}

impl MockMinesweeper {
    pub fn new(&mut config: Config) -> Self {
        let state = MinesweeperState::new(config);
        let mut bombs = vec![false; config.size()];
        for _ in 0..config.mines() {
            loop {
                let idx = config.random_index();
                if idx != config.center() && !bombs[idx] {
                    bombs[idx] = true;
                    break;
                }
            }
        }
        Self { state, bombs }
    }

    fn _reveal(&mut self, idx: Index, squares: &[Square]) {
        debug_assert!(matches!(self.state.get(idx), Status::Marked | Status::Unknown));
        if !self.bombs[idx] {
            let square = squares[idx];
            let count = square.iter().filter(|&&cidx| self.bombs[cidx]).count();
            self.state.set_known(idx, count);
            if count != 0 {
                return;
            }
            for cidx in square {
                if matches!(self.state.get(cidx), Status::Marked | Status::Unknown) {
                    self._reveal(cidx, squares);
                }
            }
        }
    }
}

impl Minesweeper for MockMinesweeper {
    fn get_bombs(&self) -> Option<&[bool]> {
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

    fn reveal(&mut self, idx: usize, squares: &[Square]) -> MsResult<()> {
        (!self.bombs[idx])
            .then(|| self._reveal(idx, squares))
            .ok_or(MinesweeperError::RevealedBomb(idx))
    }

    fn set_internal(&mut self, state: MinesweeperState) -> MsResult<()> {
        self.state = state;
        Ok(())
    }
}
