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

use std::cmp::{max, min};

use ordered_float::NotNan;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MinesweeperState {
    board: Vec<Status>,
    width: usize,
    length: usize,
    mines: usize,
}

impl MinesweeperState {
    pub fn new(width: usize, length: usize, mines: usize) -> MsResult<Self> {
        (width * length > mines)
            .then(|| Self {
                board: vec![Status::Unknown; width * length],
                width,
                length,
                mines,
            })
            .ok_or(MinesweeperError::AllCellsAreMines)
    }

    #[inline]
    pub fn board(&self) -> &Vec<Status> {
        &self.board
    }

    #[inline]
    pub fn width(&self) -> usize {
        self.width
    }

    #[inline]
    pub fn length(&self) -> usize {
        self.length
    }

    #[inline]
    pub fn mines(&self) -> usize {
        self.mines
    }

    #[inline]
    pub fn count(&self, status: Status) -> usize {
        // Cache?
        self.board.iter().filter(|status_| status_ == &&status).count()
    }

    #[inline]
    pub fn size(&self) -> usize {
        // Alias for self.board().len()
        self.width * self.length
    }

    #[inline]
    pub fn as_rc(&self, idx: usize) -> (usize, usize) {
        (idx / self.length, idx % self.length)
    }

    #[inline]
    pub fn from_rc(&self, row: usize, col: usize) -> usize {
        row * self.length + col
    }

    #[inline]
    pub fn center(&self) -> usize {
        self.from_rc(self.width / 2, self.length / 2)
    }

    pub fn square(&self, idx: usize) -> impl Iterator<Item = usize> {
        let len = self.length; // Copy
        let (row, col) = self.as_rc(idx);
        (max(1, row) - 1..=min(self.width - 1, row + 1))
            .flat_map(move |r| (max(1, col) - 1..=min(len - 1, col + 1)).map(move |c| r * len + c))
    }

    pub fn square_of(&self, idx: usize, status: Status) -> impl Iterator<Item = usize> + '_ {
        self.square(idx).filter(move |cidx| self.board[*cidx] == status)
    }

    pub fn set_known(&mut self, idx: usize, bombs: &Vec<bool>) -> MsResult<()> {
        match self.board[idx] {
            Status::Flagged => unreachable!("{} is flagged but not bomb", idx),
            Status::Known(_) => Ok(()),
            _ => {
                let count = self.square(idx).filter(|cidx| bombs[*cidx]).count();
                self.board[idx] = Status::Known(count);
                if count == 0 {
                    // Should have no flags as count is zero
                    for cidx in self.square(idx) {
                        self.set_known(cidx, bombs)?;
                    }
                }
                Ok(())
            }
        }
    }

    pub fn make_consistent(&mut self, idx: usize) -> MsResult<()> {
        if let Status::Known(count) = self.board[idx] {
            let unknowns = self.square_of(idx, Status::Unknown).collect::<Vec<usize>>();
            let minimum = self.square_of(idx, Status::Flagged).count();
            let maximum = minimum + unknowns.len();
            for cidx in unknowns.iter() {
                if count < minimum || count > maximum {
                    return Err(MinesweeperError::NumberOfMinesOutOfRange);
                } else if count == minimum {
                    self.board[*cidx] = Status::Marked;
                } else if count == maximum {
                    self.board[*cidx] = Status::Flagged;
                    for ccidx in self.square(*cidx) {
                        self.make_consistent(ccidx)?;
                    }
                }
            }
        }
        Ok(())
    }

    pub fn step(&mut self) -> MsResult<()> {
        for idx in 0..self.size() {
            self.make_consistent(idx)?;
        }
        let mut is_grouped = self.board
            .iter()
            .enumerate()
            .map(|(idx, status)| match status {
                Status::Known(x) => x != &0 && self.square_of(idx, Status::Unknown).count() != 0,
                Status::Unknown => false,
                _ => true,
            })
            .collect::<Vec<bool>>();
        let mut unknowns_groups: Vec<Vec<usize>> = Vec::new();
        for (idx, status) in self.board.iter().enumerate() {
            if let Status::Known(x) = status {
                if x != &0 && self.square_of(idx, Status::Unknown).count() != 0 && !is_grouped[idx] {
                    let mut group = Vec::new();
                    self.set_group(idx, unknowns_groups.len(), &mut group, &mut is_grouped);
                    unknowns_groups.push(group);
                }
            }
        }
        log::trace!("{:?}", is_grouped);
        Ok(())
    }

    fn set_group(&self, idx: usize, id: usize, group: &mut Vec<usize>, is_grouped: &mut Vec<bool>) {
        if is_grouped[idx] {
            return;
        }
        is_grouped[idx] = true;
        match self.board[idx] {
            Status::Known(_) => {
                for cidx in self.square(idx) {
                    self.set_group(cidx, id, group, is_grouped);
                }
            },
            Status::Unknown => {
                for cidx in self.square(idx) {
                    if self.board[cidx] != Status::Unknown {
                        self.set_group(cidx, id, group, is_grouped);
                    }
                }
            },
            _ => unreachable!("Ineligible for grouping"),
        }
    }

    pub fn fast_search(&self) -> Option<(usize, f64)> {
        if let Some(idx) = self.pos_of(Status::Marked) {
            Some((idx, 0.0))
        } else {
            let center = self.center();
            match self.board[center] {
                Status::Known(_) => None,
                Status::Flagged => unreachable!("Center cannot be bomb"),
                _ => Some((center, 0.0)),
            }
        }
    }

    pub fn slow_search(&self) -> Option<(usize, f64)> {
        if let Some(idx) = self.pos_of(Status::Marked) {
            Some((idx, 0.0))
        } else {
            let unflags = self.mines() - self.count(Status::Flagged);
            let unknowns = self.count(Status::Unknown);
            let base_prob = NotNan::new((unflags as f64) / (unknowns as f64)).ok()?;
            let mut prob = vec![None; self.size()];
            for (idx, status) in self.board.iter().enumerate() {
                if let Status::Known(count) = status {
                    let square_unknowns = self.square_of(idx, Status::Unknown).count();
                    let p = NotNan::new((*count as f64) / (square_unknowns as f64)).ok();
                    for idx_sq in self.square(idx) {
                        prob[idx_sq] = max(prob[idx_sq], p);
                    }
                }
            }
            self
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

    fn pos_of(&self, status: Status) -> Option<usize> {
        self.board.iter().position(|status_| status_ == &status)
    }
}
