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

use std::cmp::max;

use ordered_float::NotNan;

pub trait Minesweeper: Sized {
    // These getters/setters needed for abstraction
    fn get(&self) -> &MinesweeperState;
    fn get_bombs(&self) -> Option<&Vec<bool>>;
    fn set(&mut self, state: MinesweeperState);

    // Depends on implementation
    fn reveal(&mut self, idx: usize) -> MsResult<()>;

    fn solve(&mut self) -> MsResult<()> {
        while let Some((idx, p)) = self.solve_next() {
            log::debug!("Guess {:?}: {}", self.get().as_rc(idx), p);
            self.reveal(idx)?;
            log::trace!("{}", Show(self));
        }
        Ok(())
    }

    // 1.0f64 is exact
    fn solve_next(&mut self) -> Option<(usize, f64)> {
        let mut state = self.get().clone();
        if let Some(idx) = state
            .board
            .iter()
            .position(|status| status == &Status::Marked)
        {
            return Some((idx, 1.0));
        }
        let center = state.center();
        match state.board[center] {
            Status::Known(_) => (),
            Status::Flagged => unreachable!("Center cannot be bomb"),
            _ => return Some((center, 1.0)),
        }
        loop {
            let reduced = state.reduce();
            if reduced == state {
                break;
            } else {
                state = reduced;
            }
        }
        self.set(state.clone());
        if let Some(idx) = state
            .board
            .iter()
            .position(|status| status == &Status::Marked)
        {
            return Some((idx, 1.0));
        }

        let unflags = state.mines() - state.count(Status::Flagged);
        let unknowns = state.count(Status::Unknown);
        let base_prob = NotNan::new((unflags as f64) / (unknowns as f64)).ok()?;
        let mut prob = vec![None; state.size()];
        for (idx, status) in state.board.iter().enumerate() {
            if let Status::Known(count) = status {
                let square_unknowns = state.square_of(idx, Status::Unknown).count();
                let p = NotNan::new((*count as f64) / (square_unknowns as f64)).ok();
                for idx_sq in state.square(idx) {
                    prob[idx_sq] = max(prob[idx_sq], p);
                }
            }
        }
        state
            .board
            .iter()
            .enumerate()
            .filter_map(|(idx, status)| {
                (status == &Status::Unknown).then(|| (idx, prob[idx].unwrap_or(base_prob)))
            })
            .min_by_key(|(_, p)| *p)
            .map(|(idx, p)| (idx, p.into_inner()))
    }
}
