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
use std::ops::Add;

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

struct Evaluation {
    conf_counts: Vec<usize>,
    mark_counts: Vec<Vec<usize>>,
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

type Index = usize;

fn zip_with<T, F: Fn(T, T) -> T>(a: Vec<T>, b: Vec<T>, f: F) -> Vec<T> {
    a.into_iter().zip(b.into_iter()).map(|(x, y)| f(x, y)).collect()
}

fn ncr(n: usize, r: usize) -> f64 {
    let d = (n - r) as f64;
    (1..=r).map(|x| (1.0 + d / (x as f64))).product::<f64>()
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
    pub fn center(&self) -> Index {
        self.from_rc(self.width / 2, self.length / 2)
    }

    pub fn square(&self, idx: Index) -> impl Iterator<Item = Index> {
        let len = self.length; // Copy
        let (row, col) = self.as_rc(idx);
        (max(1, row) - 1..=min(self.width - 1, row + 1))
            .flat_map(move |r| (max(1, col) - 1..=min(len - 1, col + 1)).map(move |c| r * len + c))
    }

    pub fn square_of(&self, idx: Index, status: Status) -> impl Iterator<Item = Index> + '_ {
        self.square(idx)
            .filter(move |cidx| self.board[*cidx] == status)
    }

    pub fn step(&mut self) -> MsResult<()> {
        for idx in 0..self.size() {
            self.make_consistent(idx)?;
        }
        Ok(())
    }

    pub fn get_known(&self, idx: Index) -> Option<usize> {
        match self.board[idx] {
            Status::Known(x) => Some(x),
            _ => None,
        }
    }

    pub fn set_known(&mut self, idx: Index, bombs: &Vec<bool>) -> MsResult<()> {
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

    fn make_consistent(&mut self, idx: Index) -> MsResult<()> {
        if let Status::Known(count) = self.board[idx] {
            let unknowns = self.square_of(idx, Status::Unknown).collect::<Vec<Index>>();
            let minimum = self.square_of(idx, Status::Flagged).count();
            let maximum = minimum + unknowns.len();
            if !(minimum..=maximum).contains(&count) {
                return Err(MinesweeperError::NumberOfMinesOutOfRange);
            }
            for cidx in unknowns.iter() {
                if count == minimum {
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

    fn pos_of(&self, status: Status) -> Option<Index> {
        self.board.iter().position(|status_| status_ == &status)
    }

    fn set(&mut self, idx: Index, status: Status) -> MsResult<()> {
        self.board[idx] = status;
        for cidx in self.square(idx) {
            self.make_consistent(cidx)?;
        }
        Ok(())
    }

    fn evaluate(&self, group_idx: usize, group: &Vec<Index>) -> Option<Evaluation> {
        if let Some(idx) = group.get(group_idx) {
            match self.board[*idx] {
                Status::Flagged | Status::Marked => self.evaluate(group_idx + 1, group),
                Status::Known(_) => unreachable!(),
                Status::Unknown => {
                    let mut clf = self.clone();
                    let mut clm = self.clone();
                    match (
                        clf.set(*idx, Status::Flagged)
                            .ok()
                            .and_then(|_| clf.evaluate(group_idx + 1, group)),
                        clm.set(*idx, Status::Marked)
                            .ok()
                            .and_then(|_| clm.evaluate(group_idx + 1, group)),
                    ) {
                        (Some(e_0), Some(e_1)) => Some(Evaluation {
                            conf_counts: zip_with(e_0.conf_counts, e_1.conf_counts, usize::add),
                            mark_counts: zip_with(e_0.mark_counts, e_1.mark_counts, |u, v| zip_with(u, v, usize::add)),
                        }),
                        (x, y) => x.or(y)
                    }
                }
            }
        } else {
            let zeros = vec![0; group.len() + 1];
            let mut one_hot = zeros.clone();
            let flags_count = group.iter().filter(|idx| self.board[**idx] == Status::Flagged).count();
            one_hot[flags_count] = 1;
            let mark_counts = group.iter().map(|idx| match self.board[*idx] {
                Status::Flagged => zeros.clone(),
                Status::Marked => one_hot.clone(),
                _ => unreachable!()
            }).collect();
            Some(Evaluation { conf_counts: one_hot, mark_counts })
        }
    }

    fn set_group(&self, idx: Index, group: &mut Vec<Index>, assigned: &mut Vec<bool>) {
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
            _ => Some((1.0, center)),
        }
    }

    pub fn fast_search(&self) -> Option<ProbWithIndex> {
        self.pos_of(Status::Marked)
            .map(|idx| (1.0, idx))
    }

    #[allow(dead_code)]
    pub fn crude_search(&self) -> Option<ProbWithIndex> {
        let flags_remaining_in_board = self.mines() - self.count(Status::Flagged);
        let unknowns_remaining_in_board = self.count(Status::Unknown);
        let base = R64::try_new((flags_remaining_in_board as f64) / (unknowns_remaining_in_board as f64))?;
        let mut compls = vec![None; self.size()];
        for (idx, status) in self.board.iter().enumerate() {
            if let Status::Known(count) = status {
                let square_unknowns = self.square_of(idx, Status::Unknown).count();
                let compl = R64::try_new((*count as f64) / (square_unknowns as f64));
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
                    .then(|| (idx, R64::try_new(1.0).unwrap() - compls[idx].unwrap_or(base)))
            })
            .max()
            .map(|(idx, p)| (p.raw(), idx))
    }

    fn estimate(&self, group: Vec<Index>) -> Option<(R64, Index)> {
        group
            .into_iter()
            .filter_map(|idx| {
                self.square(idx)
                    .filter_map(|cidx| {
                        self.get_known(cidx).and_then(|x| {
                            let flaggeds = self.square_of(cidx, Status::Flagged).count();
                            let unknowns = self.square_of(cidx, Status::Unknown).count();
                            R64::try_new(1.0 - ((x - flaggeds) as f64) / (unknowns as f64))
                        })
                    })
                    .reduce(|a, b| a * b)
                    .map(|p| (p, idx))
            })
            .max()
    }

    fn brute_force(&mut self, group: &Vec<Index>, eval: &Evaluation) {
        let Evaluation { conf_counts, mark_counts } = eval;
        for (group_idx, mark_count) in mark_counts.iter().enumerate() {
            let cell = &mut self.board[group[group_idx]];
            if mark_count.iter().all(|x| x == &0) {
                *cell = Status::Flagged;
            } else if mark_count == conf_counts {
                *cell = Status::Marked;
            }
        }
    }
    
    fn probabilistic_search(&self, group: &Vec<Index>, eval: &Evaluation) -> Option<(R64, Index)> {
        let Evaluation { conf_counts, mark_counts } = eval;
        let unknowns_otherwise = group.iter().filter(|idx| self.board[**idx] == Status::Unknown).count();
        let unknowns_remaining_in_board = self.count(Status::Unknown) - unknowns_otherwise;
        let flags_remaining_in_board = self.mines() - self.count(Status::Flagged);
        let base = (flags_remaining_in_board as f64) / (unknowns_remaining_in_board as f64);
        mark_counts
            .iter()
            .enumerate()
            .map(|(group_idx, mark_count)| {
                let idx = group[group_idx];
                let mark_prob = mark_count.iter()
                .zip(conf_counts.iter())
                .enumerate()
                .take(unknowns_otherwise + 1)
                .filter_map(|(i, (mark, conf))| {
                    if self.board[idx] != Status::Unknown {
                        return None;
                    }
                    let p = (*mark as f64) / (*conf as f64)
                        * ncr(unknowns_otherwise, i)
                        * base.powi(i as i32)
                        * (1.0 - base).powi((unknowns_otherwise - i) as i32);
                    R64::try_new(p)
                })
                .sum::<R64>();
                (mark_prob, idx)

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
                            let eval = self.evaluate(0, &group).expect("Valid assignment exists");
                            self.brute_force(&group, &eval);
                            self.fast_search()
                                .map(|(p, idx)| (R64::try_new(p).unwrap(), idx))
                                .or_else(|| self.probabilistic_search(&group, &eval))
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
        let base = R64::try_new(0f64).unwrap(); // R64::try_new((self.mines() as f64) / (self.count(Status::Unknown) as f64)).unwrap();
        max(best, ridx.map(|idx| (base, idx))).map(|(p, idx)| (p.raw(), idx))
    }
}
