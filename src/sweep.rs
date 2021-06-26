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

pub trait Minesweeper {
    fn get_bombs(&self) -> Option<&Vec<bool>>;
    fn get_state(&self) -> &MinesweeperState;
    fn pull(&mut self) -> MsResult<MinesweeperState>;
    fn flag(&mut self, idx: usize) -> MsResult<()>;
    fn reveal(&mut self, idx: usize) -> MsResult<()>;
    fn set_internal(&mut self, state: MinesweeperState) -> MsResult<()>;

    fn step(&mut self) -> MsResult<Option<(usize, f64)>> {
        let state = self.pull()?;
        if let Some((idx, p)) = state.fast_search() {
            self.reveal(idx)?;
            Ok(Some((idx, p)))
        } else {
            let next_state = state.step();
            let info = next_state.slow_search();
            for idx in state
                .board()
                .iter()
                .zip(next_state.board().iter())
                .enumerate()
                .filter_map(|(idx, (prev, next))| {
                    (prev != &Status::Flagged && next == &Status::Flagged)
                    .then(|| idx)
                })
            {
                self.flag(idx)?;
            }
            self.set_internal(next_state)?;
            if let Some((idx, _)) = info {
                self.reveal(idx)?;
            }
            Ok(info)
        }  
    }
}
