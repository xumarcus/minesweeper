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
            .ok_or(MinesweeperError::NumberOfMinesOutOfRange)
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

    pub fn step(&mut self) -> MsResult<()> {
        for idx in 0..self.size() {
            self.make_consistent(idx)?;
        }
        Ok(())
    }

    pub fn get_known(&self, idx: usize) -> Option<usize> {
        match self.board[idx] {
            Status::Known(x) => Some(x),
            _ => None
        }
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

    fn pos_of(&self, status: Status) -> Option<usize> {
        self.board.iter().position(|status_| status_ == &status)
    }

    // no allocation for Vec::new
    fn evaluate(&self, group: &[usize]) -> (usize, Vec<(usize, usize)>) {
        if let Some((idx, rest)) = group.split_first() {
            let mut clf = self.clone();
            clf.board[*idx] = Status::Flagged;
            let mut clm = self.clone();
            clm.board[*idx] = Status::Marked;
            match (
                clf.make_consistent(*idx).map(|_| clf.evaluate(rest)),
                clm.make_consistent(*idx).map(|_| clm.evaluate(rest)),
            ) {
                (Ok((a, u)), Ok((b, v))) => {
                    let mut w = u.into_iter()
                        .zip(v.into_iter())
                        .map(|((c, i), (d, i_))| {
                            debug_assert_eq!(i, i_);
                            (c + d, i)
                        })
                        .collect::<Vec<(usize, usize)>>();
                    w.push((b, *idx));
                    (a + b, w)
                },
                (Ok((a, mut u)), _) => {
                    u.push((0, *idx));
                    (a, u)
                },
                (_, Ok((b, mut v))) => {
                    v.push((b, *idx));
                    (b, v)
                },
                _ => (0, Vec::new())
            }
        } else {
            (1, Vec::new()) 
        }
    }

    fn set_group(&self, idx: usize, group: &mut Vec<usize>, assigned: &mut Vec<bool>) {
        if assigned[idx] {
            return;
        }
        assigned[idx] = true;
        if self.board[idx] == Status::Unknown {
            group.push(idx);
        }
        match self.board[idx] {
            Status::Known(_) => {
                for cidx in self.square(idx) {
                    self.set_group(cidx, group, assigned);
                }
            },
            Status::Unknown => {
                for cidx in self.square(idx) {
                    if self.board[cidx] != Status::Unknown {
                        self.set_group(cidx, group, assigned);
                    }
                }
            },
            _ => unreachable!("Ineligible for grouping"),
        }
    }

    
    pub fn center_search(&self) -> Option<ProbWithIndex> {
        let center = self.center();
        match self.board[center] {
            Status::Known(_) => None,
            Status::Flagged => unreachable!("Center cannot be bomb"),
            _ => Some((Rational::from_integer(1), center)),
        }
    }

    pub fn fast_search(&self) -> Option<ProbWithIndex> {
        self
        .pos_of(Status::Marked)
        .map(|idx| (Rational::from_integer(1), idx))
    }

    #[allow(dead_code)]
    pub fn crude_search(&self) -> Option<ProbWithIndex> {
        let unknowns = self.count(Status::Unknown);
        let base = (unknowns != 0).then(|| Rational::new(self.mines() - self.count(Status::Flagged), unknowns))?;
        let mut compls = vec![None; self.size()];
        for (idx, status) in self.board.iter().enumerate() {
            if let Status::Known(count) = status {
                let square_unknowns = self.square_of(idx, Status::Unknown).count();
                let compl = (square_unknowns != 0).then(|| Rational::new(*count, square_unknowns));
                for idx_sq in self.square(idx) {
                    compls[idx_sq] = max(compls[idx_sq], compl);
                }
            }
        }
        self
        .board
        .iter()
        .enumerate()
        .filter_map(|(idx, status)| {
            (status == &Status::Unknown).then(|| (idx, Rational::from_integer(1) - compls[idx].unwrap_or(base)))
        })
        .max()
        .map(|(idx, p)| (p, idx))
    }
    
    pub fn slow_search(&mut self) -> Option<ProbWithIndex> {
        let mut assigned = self.board
            .iter()
            .enumerate()
            .map(|(idx, status)| match status {
                Status::Known(x) => !(x != &0 && self.square_of(idx, Status::Unknown).count() != 0),
                Status::Unknown => false,
                _ => true,
            })
            .collect::<Vec<bool>>();
        self.clone().board
        .iter()
        .enumerate()
        .filter_map(|(idx, status)| {
            (matches!(status, Status::Known(_)) && !assigned[idx]).then(|| {
                let mut group = Vec::new();
                self.set_group(idx, &mut group, &mut assigned);
                log::trace!("{:?}", group.iter().map(|&idx| self.as_rc(idx)).rev().collect::<Vec<_>>());
                if group.len() <= 16 {
                    let (max_count, cells) = self.evaluate(&group);
                    log::trace!("{:?}", cells);
                    debug_assert_ne!(max_count, 0);
                    debug_assert_eq!(cells.len(), group.len());
                    cells
                    .into_iter()
                    .map(|(marked_count, idx)| {
                        if marked_count == 0 {
                            self.board[idx] = Status::Flagged;
                        } else if marked_count == max_count {
                            self.board[idx] = Status::Marked;
                        }
                        (Rational::new(marked_count, max_count), idx)
                    })
                    .max()
                } else {
                    group.
                    into_iter()
                    .filter_map(|idx| self
                        .square(idx)
                        .filter_map(|cidx| self
                            .get_known(cidx)
                            .and_then(|x| {
                                let flaggeds = self.square_of(cidx, Status::Flagged).count();
                                let unknowns = self.square_of(cidx, Status::Unknown).count();
                                (unknowns != 0).then(|| Rational::from_integer(1) - Rational::new(x - flaggeds, unknowns))
                            })
                        )
                        .reduce(|a, b| a * b)
                        .map(|p| (p, idx))
                    )
                    .max()
                }
            }).flatten()
        })
        .max()
    }
}
