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
// but WITHOUT ANY WARRANTY; without   the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with minesweeper.  If not, see <http://www.gnu.org/licenses/>.

use super::*;

pub trait Minesweeper {
    fn get_bombs(&self) -> Option<&[bool]>;
    fn get_config(&self) -> &Config;
    fn get_state(&self) -> &MinesweeperState;
    fn pull(&self) -> MsResult<MinesweeperState>;
    fn flag(&mut self, idx: usize) -> MsResult<()>;
    fn reveal(&mut self, idx: usize) -> MsResult<()>;
    fn set_internal(&mut self, state: MinesweeperState) -> MsResult<()>;
    fn push(&mut self, state: MinesweeperState) -> MsResult<()> {
        let indices = self
            .get_state()
            .board()
            .iter()
            .zip(state.board().iter())
            .enumerate()
            .filter_map(|(idx, (p, n))| {
                (p != &Status::Flagged && n == &Status::Flagged).then(|| idx)
            })
            .collect::<Vec<Index>>();
        for idx in indices {
            self.flag(idx)?;
        }
        self.set_internal(state)
    }
    fn solve_next(&mut self, solver: &Solver) -> MsResult<Option<ScoredIndex>> {
        let mut state = self.pull()?;
        let scored_index = solver.solve_next(&mut state);
        log::info!("{:?}", scored_index);
        self.push(state)?;
        if let Some((_, idx)) = scored_index {
            self.reveal(idx)?;
        }
        Ok(scored_index)
    }
}
