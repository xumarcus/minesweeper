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

fn enforce_consistency(state: &MinesweeperState) -> MsResult<MinesweeperState> {
    let mut next = state.clone();
    next.step()?;
    Ok(next)
}

pub trait Minesweeper {
    fn get_bombs(&self) -> Option<&Vec<bool>>;
    fn get_state(&self) -> &MinesweeperState;
    fn pull(&mut self) -> MsResult<MinesweeperState>;
    fn flag(&mut self, idx: usize) -> MsResult<()>;
    fn reveal(&mut self, idx: usize) -> MsResult<()>;
    fn set_internal(&mut self, state: MinesweeperState) -> MsResult<()>;
    fn flag_all(&mut self, state: &MinesweeperState, next: &MinesweeperState) -> MsResult<()> {
        let iter = state
            .board()
            .iter()
            .zip(next.board().iter())
            .enumerate()
            .filter_map(|(idx, (p, n))| {
                (p != &Status::Flagged && n == &Status::Flagged).then(|| idx)
            });
        for idx in iter {
            self.flag(idx)?;
        }
        Ok(())
    }
    fn step(&mut self) -> MsResult<Option<ProbWithIndex>> {
        let state = self.pull()?;
        if let Some((p, idx)) = state.center_search() {
            self.reveal(idx)?;
            return Ok(Some((p, idx)));
        }
        if let Some((p, idx)) = state.fast_search() {
            self.reveal(idx)?;
            return Ok(Some((p, idx)));
        }
        let mut next_state = enforce_consistency(&state)?;
        let info = next_state
            .fast_search()
            .or_else(|| next_state.slow_search());
        self.flag_all(&state, &next_state)?;
        self.set_internal(next_state)?;
        if let Some((_, idx)) = info {
            self.reveal(idx)?;
        }
        Ok(info)
    }
}
