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

#[derive(Clone, Copy, Debug)]
pub struct Config {
    pub width: usize,
    pub length: usize,
    pub mines: usize,
}

impl Config {
    pub fn new(width: usize, length: usize, mines: usize) -> MsResult<Self> {
        (width * length > mines)
            .then(|| Self {
                width,
                length,
                mines,
            })
            .ok_or(MinesweeperError::InvalidParameters)
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

    pub fn square_filter_status<'a>(
        &self,
        board: &'a Vec<Status>,
        idx: usize,
        status: Status,
    ) -> impl Iterator<Item = usize> + 'a {
        self.square(idx).filter(move |cidx| board[*cidx] == status)
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
}
