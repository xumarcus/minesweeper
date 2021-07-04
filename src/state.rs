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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MinesweeperState {
    board: Vec<Status>,
    flags_remaining: usize,
    unknowns: usize,
    knowns: usize,
}

impl MinesweeperState {
    pub fn new(config: &Config) -> Self {
        let board = vec![Status::Unknown; config.size()];
        Self {
            board,
            flags_remaining: config.mines(),
            unknowns: config.size(),
            knowns: 0,
        }
    }

    pub fn board(&self) -> &[Status] {
        &self.board
    }

    #[inline]
    pub fn flags_remaining(&self) -> usize {
        self.flags_remaining
    }

    #[inline]
    pub fn knowns(&self) -> usize {
        self.knowns
    }

    #[inline]
    pub fn unknowns(&self) -> usize {
        self.unknowns
    }

    #[inline]
    pub fn size(&self) -> usize {
        self.board.len()
    }

    #[inline]
    pub fn set_flag(&mut self, idx: Index) -> bool {
        debug_assert_eq!(self.board[idx], Status::Unknown);
        if self.unknowns > 0 && self.flags_remaining() > 0 {
            self.board[idx] = Status::Flagged;
            self.unknowns -= 1;
            self.flags_remaining -= 1;
            true
        } else {
            false
        }
    }

    #[inline]
    pub fn set_mark(&mut self, idx: Index) -> bool {
        debug_assert_eq!(self.board[idx], Status::Unknown);
        self.board[idx] = Status::Marked;
        if self.unknowns > 0 {
            self.unknowns -= 1;
            true
        } else {
            false
        }
    }

    pub fn filter_status<'a>(
        &'a self,
        square: &'a Square,
        status: Status,
    ) -> impl Iterator<Item = Index> + 'a {
        square
            .iter()
            .filter_map(move |&cidx| (self.board[cidx] == status).then(|| cidx))
    }

    #[inline]
    pub fn get(&self, idx: Index) -> Status {
        self.board[idx]
    }

    #[inline]
    pub fn get_known(&self, idx: Index) -> Option<usize> {
        match self.board[idx] {
            Status::Known(x) => Some(x),
            _ => None,
        }
    }

    pub fn reveal(&mut self, idx: Index, bombs: &[bool], config: Config) {
        debug_assert!(matches!(self.get(idx), Status::Marked | Status::Unknown));
        if !bombs[idx] {
            let count = config.square(idx).filter(|&cidx| bombs[cidx]).count();
            if self.board[idx] == Status::Unknown {
                self.unknowns -= 1;
            }
            self.board[idx] = Status::Known(count);
            self.knowns += 1;
            if count != 0 {
                return;
            }
            for cidx in config.square(idx) {
                if matches!(self.get(cidx), Status::Marked | Status::Unknown) {
                    self.reveal(cidx, bombs, config);
                }
            }
        }
    }
}
