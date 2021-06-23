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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MinesweeperState {
    pub board: Vec<Status>,
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
            .ok_or(MinesweeperError::InvalidParameters)
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
        self.width * self.length
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

    pub fn reduce(&self) -> Self {
        let mut cl = self.clone();
        for (idx, status) in self.board.iter().enumerate() {
            if let Status::Known(count) = status {
                let unknowns = self.square_of(idx, Status::Unknown).collect::<Vec<usize>>();
                let sq_flags = self.square_of(idx, Status::Flagged).count();
                for cidx in unknowns.iter() {
                    if *count == sq_flags {
                        cl.board[*cidx] = Status::Marked;
                    } else if *count == unknowns.len() + sq_flags {
                        cl.board[*cidx] = Status::Flagged;
                    }
                }
            }
        }
        cl
    }
}
