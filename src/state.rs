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

pub struct ShowState<'a> {
    state: &'a MinesweeperState,
    bombs: Option<&'a Vec<bool>>,
}

impl<'a> ShowState<'a> {
    pub fn new(state: &'a MinesweeperState, bombs: Option<&'a Vec<bool>>) -> Self {
        Self { state, bombs }
    }
}

impl<'a> fmt::Display for ShowState<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let state = &self.state;
        writeln!(f, "Dimensions: {} x {}", state.width(), state.length())?;
        writeln!(
            f,
            "Flagged: {} / {}",
            state.count(Status::Flagged),
            state.mines()
        )?;
        for (idx, status) in state.board().iter().enumerate() {
            if idx % state.length() == 0 {
                write!(f, "\n")?;
            }
            if *self
                .bombs
                .and_then(|bombs| bombs.get(idx))
                .unwrap_or(&false)
            {
                match status {
                    Status::Flagged => write!(f, "🚩")?,
                    Status::Known(_) => unreachable!("Is bomb"),
                    Status::Marked => unreachable!("Wrong solution"),
                    Status::Unknown => write!(f, "💣")?,
                }
            } else {
                match status {
                    Status::Flagged => write!(f, "🏁")?,
                    Status::Known(x) => write!(f, "{}.", x)?,
                    Status::Marked => write!(f, "✅")?,
                    Status::Unknown => write!(f, "❔")?,
                }
            }
        }
        Ok(())
    }
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
        self.board
            .iter()
            .filter(|status_| status_ == &&status)
            .count()
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
        self.square(idx)
            .filter(move |cidx| self.board[*cidx] == status)
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
            _ => None,
        }
    }

    pub fn set_known(&mut self, idx: usize, bombs: &Vec<bool>) -> MsResult<()> {
        match self.board[idx] {
            Status::Flagged => Err(MinesweeperError::FlaggedButNotBomb(idx)),
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

    fn make_consistent(&mut self, idx: usize) -> MsResult<()> {
        if let Status::Known(count) = self.board[idx] {
            let unknowns = self.square_of(idx, Status::Unknown).collect::<Vec<usize>>();
            let minimum = self.square_of(idx, Status::Flagged).count();
            let maximum = minimum + unknowns.len();
            for cidx in unknowns.iter() {
                if !(minimum..=maximum).contains(&count) {
                    return Err(MinesweeperError::NumberOfMinesOutOfRange);
                } else if count == minimum {
                    self.board[*cidx] = Status::Marked;
                } else if count == maximum {
                    self.board[*cidx] = Status::Flagged;
                } else {
                    continue;
                }
                for ccidx in self.square(*cidx) {
                    self.make_consistent(ccidx)?;
                }
            }
        }
        Ok(())
    }

    fn pos_of(&self, status: Status) -> Option<usize> {
        self.board.iter().position(|status_| status_ == &status)
    }

    fn set(&mut self, idx: usize, status: Status) -> MsResult<()> {
        self.board[idx] = status;
        for cidx in self.square(idx) {
            self.make_consistent(cidx)?;
        }
        Ok(())
    }

    fn evaluate(&self, group: &[usize]) -> Option<(usize, Vec<(usize, usize)>)> {
        if let Some((idx, rest)) = group.split_first() {
            match self.board[*idx] {
                Status::Flagged => {
                    let (a, mut u) = self.evaluate(rest)?;
                    u.push((0, *idx));
                    Some((a, u))
                }
                Status::Marked => {
                    let (b, mut v) = self.evaluate(rest)?;
                    v.push((b, *idx));
                    Some((b, v))
                }
                Status::Known(_) => unreachable!(),
                Status::Unknown => {
                    let mut clf = self.clone();
                    let mut clm = self.clone();
                    match (
                        clf.set(*idx, Status::Flagged /**/)
                            .ok()
                            .and_then(|_| clf.evaluate(rest)),
                        clm.set(*idx, Status::Marked /* */)
                            .ok()
                            .and_then(|_| clm.evaluate(rest)),
                    ) {
                        (Some((a, u)), Some((b, v))) => {
                            if u.len() != v.len() {
                                unreachable!("{:?} {:?}", u, v);
                            }
                            let mut w = u
                                .into_iter()
                                .zip(v.into_iter())
                                .map(|((c, i), (d, i_))| {
                                    debug_assert_eq!(i, i_);
                                    (c + d, i)
                                })
                                .collect::<Vec<(usize, usize)>>();
                            w.push((b, *idx));
                            Some((a + b, w))
                        }
                        (Some((a, mut u)), _) => {
                            u.push((0, *idx));
                            Some((a, u))
                        }
                        (_, Some((b, mut v))) => {
                            v.push((b, *idx));
                            Some((b, v))
                        }
                        _ => None,
                    }
                }
            }
        } else {
            Some((1, Vec::new()))
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
            }
            Status::Unknown => {
                for cidx in self.square(idx) {
                    if self.board[cidx] != Status::Unknown {
                        self.set_group(cidx, group, assigned);
                    }
                }
            }
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
        self.pos_of(Status::Marked)
            .map(|idx| (Rational::from_integer(1), idx))
    }

    #[allow(dead_code)]
    pub fn crude_search(&self) -> Option<ProbWithIndex> {
        let unknowns = self.count(Status::Unknown);
        let base = (unknowns != 0)
            .then(|| Rational::new(self.mines() - self.count(Status::Flagged), unknowns))?;
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
        self.board
            .iter()
            .enumerate()
            .filter_map(|(idx, status)| {
                (status == &Status::Unknown)
                    .then(|| (idx, Rational::from_integer(1) - compls[idx].unwrap_or(base)))
            })
            .max()
            .map(|(idx, p)| (p, idx))
    }

    fn estimate(&self, group: Vec<usize>) -> Option<ProbWithIndex> {
        group
            .into_iter()
            .filter_map(|idx| {
                self.square(idx)
                    .filter_map(|cidx| {
                        self.get_known(cidx).and_then(|x| {
                            let flaggeds = self.square_of(cidx, Status::Flagged).count();
                            let unknowns = self.square_of(cidx, Status::Unknown).count();
                            (unknowns != 0).then(|| {
                                Rational::from_integer(1) - Rational::new(x - flaggeds, unknowns)
                            })
                        })
                    })
                    .reduce(|a, b| a * b)
                    .map(|p| (p, idx))
            })
            .max()
    }

    fn backtrack(&mut self, group: Vec<usize>) -> Option<ProbWithIndex> {
        let (max_count, cells) = self.evaluate(&group).expect("Valid assignment exists");
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
    }

    pub fn slow_search(&mut self) -> Option<ProbWithIndex> {
        let mut assigned = self
            .board
            .iter()
            .enumerate()
            .map(|(idx, status)| match status {
                Status::Known(x) => !(x != &0 && self.square_of(idx, Status::Unknown).count() != 0),
                Status::Unknown => false,
                _ => true,
            })
            .collect::<Vec<bool>>();
        let best = self
            .clone()
            .board
            .iter()
            .enumerate()
            .filter_map(|(idx, status)| {
                (matches!(status, Status::Known(_)) && !assigned[idx])
                    .then(|| {
                        let mut group = Vec::new();
                        self.set_group(idx, &mut group, &mut assigned);
                        if group.len() <= 32 {
                            self.backtrack(group)
                        } else {
                            self.estimate(group)
                        }
                    })
                    .flatten()
            })
            .max();

        let ridx = assigned
            .iter()
            .enumerate()
            .filter_map(|(idx, is_assigned)| {
                (!is_assigned && self.board[idx] == Status::Unknown).then(|| idx)
            })
            .next();
        let base = Rational::new_raw(self.mines(), self.count(Status::Unknown));
        max(best, ridx.map(|idx| (base, idx)))
    }
}
