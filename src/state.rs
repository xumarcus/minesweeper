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
    flags: usize,
    unknowns: usize,
    knowns: usize,
}

impl MinesweeperState {
    pub fn new(width: usize, length: usize, mines: usize) -> MsResult<Self> {
        (width * length > mines)
            .then(|| {
                let board = vec![Status::Unknown; width * length];
                Self {
                    board,
                    width,
                    length,
                    mines,
                    flags: 0,
                    unknowns: width * length,
                    knowns: 0,
                }
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
    pub fn size(&self) -> usize {
        self.board().len()
    }

    #[inline]
    pub fn mines(&self) -> usize {
        self.mines
    }

    #[inline]
    pub fn flags(&self) -> usize {
        self.flags
    }

    #[inline]
    pub fn marks(&self) -> usize {
        self.size() - self.flags() - self.unknowns() - self.knowns()
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
    pub fn flags_remaining(&self) -> usize {
        self.mines() - self.flags()
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

    #[inline]
    fn set_flag(&mut self, idx: Index) {
        debug_assert_eq!(self.board[idx], Status::Unknown);
        self.board[idx] = Status::Flagged;
        self.unknowns -= 1;
        self.flags += 1;
    }

    #[inline]
    fn set_mark(&mut self, idx: Index) {
        debug_assert_eq!(self.board[idx], Status::Unknown);
        self.board[idx] = Status::Marked;
        self.unknowns -= 1;
    }

    pub fn set_known(&mut self, idx: Index, bombs: &Vec<bool>) {
        debug_assert!(matches!(self.board[idx], Status::Marked | Status::Unknown));
        if !bombs[idx] {
            let square = self.square(idx);
            let count = square.iter().filter(|&&cidx| bombs[cidx]).count();
            if self.board[idx] == Status::Unknown {
                self.unknowns -= 1;
            }
            self.board[idx] = Status::Known(count);
            self.knowns += 1;
            if count == 0 {
                for cidx in square {
                    if matches!(self.board[cidx], Status::Marked | Status::Unknown) {
                        self.set_known(cidx, bombs);
                    }
                }
            }
        }
    }

    fn square(&self, idx: Index) -> Square {
        let (row, col) = self.as_rc(idx);
        let rmin = max(1, row) - 1;
        let rmax = min(self.width - 1, row + 1);
        let cmin = max(1, col) - 1;
        let cmax = min(self.length - 1, col + 1);
        (rmin..=rmax)
            .flat_map(|r| (cmin..=cmax).map(move |c| self.from_rc(r, c)))
            .filter(|cidx| cidx != &idx)
            .collect()
    }

    fn filter_status<'a>(
        &'a self,
        square: &'a Square,
        status: Status,
    ) -> impl Iterator<Item = Index> + 'a {
        square
            .iter()
            .filter_map(move |&cidx| (self.board[cidx] == status).then(|| cidx))
    }

    fn get(&self, idx: Index) -> Option<usize> {
        match self.board[idx] {
            Status::Known(x) => Some(x),
            _ => None,
        }
    }

    fn make_consistent_all(&mut self) -> MsResult<()> {
        for idx in 0..self.size() {
            self.make_consistent(idx)?;
        }
        Ok(())
    }

    fn make_consistent_sq(&mut self, idx: Index) -> MsResult<()> {
        for idx in self.square(idx) {
            self.make_consistent(idx)?;
        }
        Ok(())
    }

    fn make_consistent(&mut self, idx: Index) -> MsResult<()> {
        if let Status::Known(count) = self.board[idx] {
            let square = self.square(idx);
            let minimum = self.filter_status(&square, Status::Flagged).count();
            let unknowns = self
                .filter_status(&square, Status::Unknown)
                .collect::<Square>();
            let maximum = minimum + unknowns.len();
            if !(minimum..=maximum).contains(&count) {
                return Err(MinesweeperError::NumberOfMinesOutOfRange);
            }
            for cidx in unknowns {
                if count == minimum && self.board[cidx] != Status::Marked {
                    self.set_mark(cidx);
                } else if count == maximum && self.board[cidx] != Status::Flagged {
                    self.set_flag(cidx);
                } else {
                    continue;
                }
                self.make_consistent_sq(cidx)?;
            }
        }
        Ok(())
    }

    fn evaluate(&mut self, group_idx: usize, group: &[usize]) -> Option<Evaluation> {
        if let Some(idx) = group.get(group_idx) {
            let succ = group_idx + 1;
            match self.board[*idx] {
                Status::Flagged | Status::Marked => self.evaluate(succ, group),
                Status::Known(_) => unreachable!(),
                Status::Unknown => {
                    let mut selm = self.clone();
                    self.set_flag(*idx);
                    selm.set_mark(*idx);
                    let self_eval = self
                        .make_consistent_sq(*idx)
                        .ok()
                        .and_then(|_| self.evaluate(succ, group));
                    let selm_eval = selm
                        .make_consistent_sq(*idx)
                        .ok()
                        .and_then(|_| selm.evaluate(succ, group));
                    util::lift(self_eval, selm_eval, Evaluation::add)
                }
            }
        } else {
            let size = group.len() + 1;
            let fidx = group
                .iter()
                .filter(|&&idx| self.board[idx] == Status::Flagged)
                .count();
            let mark_counts = group
                .iter()
                .map(|idx| match self.board[*idx] {
                    Status::Flagged => smallvec![0; size],
                    Status::Marked => util::one_hot(size, fidx),
                    _ => unreachable!(),
                })
                .collect();
            Some(Evaluation {
                conf_counts: util::one_hot(size, fidx),
                mark_counts,
            })
        }
    }

    fn set_group(&self, idx: Index, group: &mut Group<Index>, assigned: &mut Vec<bool>) {
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

    pub fn center_search(&self) -> Option<ScoredUnknown> {
        let center = self.center();
        match self.board[center] {
            Status::Known(_) => None,
            Status::Flagged | Status::Marked => unreachable!(),
            _ => Some((R64::new(1.0), center)),
        }
    }

    pub fn fast_search(&mut self) -> Option<ScoredUnknown> {
        let idx = self
            .board
            .iter()
            .position(|status| status == &Status::Marked)?;
        Some((R64::new(1.0), idx))
    }

    #[allow(dead_code)]
    pub fn crude_search(&self) -> Option<ScoredUnknown> {
        let base = R64::try_new((self.flags() as f64) / (self.unknowns() as f64))?;
        let mut compls = vec![None; self.size()];
        for (idx, status) in self.board.iter().enumerate() {
            if let Status::Known(count) = status {
                let square = self.square(idx);
                let unknowns = self.filter_status(&square, Status::Unknown).count();
                let compl = R64::try_new((*count as f64) / (unknowns as f64));
                for idx_sq in square {
                    compls[idx_sq] = max(compls[idx_sq], compl);
                }
            }
        }
        let (idx, p) = self
            .board
            .iter()
            .enumerate()
            .filter_map(|(idx, status)| {
                (status == &Status::Unknown).then(|| {
                    (
                        idx,
                        R64::try_new(1.0).unwrap() - compls[idx].unwrap_or(base),
                    )
                })
            })
            .max()?;
        Some((p, idx))
    }

    fn estimate(&self, group: Vec<Index>) -> Option<ScoredUnknown> {
        group
            .into_iter()
            .filter_map(|idx| {
                self.square(idx)
                    .into_iter()
                    .filter_map(|cidx| {
                        self.get(cidx).and_then(|x| {
                            let square = self.square(cidx);
                            let flaggeds = self.filter_status(&square, Status::Flagged).count();
                            let unknowns = self.filter_status(&square, Status::Unknown).count();
                            R64::try_new(1.0 - ((x - flaggeds) as f64) / (unknowns as f64))
                        })
                    })
                    .reduce(|a, b| a * b)
                    .map(|p| (p, idx))
            })
            .max()
    }

    fn estimate_unassigned(&self, group: Vec<Index>) -> Option<ScoredUnknown> {
        let idx = group.first()?;
        let p = R64::try_new((self.flags_remaining() as f64) / (self.unknowns() as f64))?; // @todo
        Some((p, *idx))
    }

    fn brute_force(&mut self, group: &Group<Index>, eval: &Evaluation) {
        let Evaluation {
            conf_counts,
            mark_counts,
        } = eval;
        for (group_idx, mark_count) in mark_counts.iter().enumerate() {
            let idx = group[group_idx];
            if mark_count.iter().all(|x| x == &0) {
                self.set_flag(idx);
            } else if mark_count == conf_counts {
                self.set_mark(idx);
            }
        }
    }

    fn probabilistic_search(
        &self,
        group: &Group<Index>,
        eval: &Evaluation,
    ) -> Option<(R64, Index)> {
        let Evaluation {
            conf_counts,
            mark_counts,
        } = eval;
        let unknowns_otherwise = group
            .iter()
            .filter(|idx| self.board[**idx] == Status::Unknown)
            .count();
        let base = R64::try_new((self.flags_remaining() as f64) / (self.unknowns() as f64))?.raw();
        debug_assert!(0.0 <= base && base <= 1.0);
        mark_counts
            .iter()
            .enumerate()
            .map(|(group_idx, mark_count)| {
                let idx = group[group_idx];
                let mark_prob = mark_count
                    .iter()
                    .zip(conf_counts.iter())
                    .enumerate()
                    .take(unknowns_otherwise + 1)
                    .filter_map(|(i, (mark, conf))| {
                        if self.board[idx] != Status::Unknown {
                            return None;
                        }
                        let p = (*mark as f64) / (*conf as f64)
                            * Binomial::new(unknowns_otherwise, base).mass(i);
                        R64::try_new(p)
                    })
                    .sum::<R64>();
                (mark_prob, idx)
            })
            .max()
    }

    pub fn slow_search(&mut self) -> Option<ScoredUnknown> {
        let mut assigned = self
            .board
            .iter()
            .enumerate()
            .map(|(idx, status)| match status {
                Status::Known(x) if x != &0 => {
                    let square = self.square(idx);
                    self.filter_status(&square, Status::Unknown).count() == 0
                }
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
                        let mut group = SmallVec::new();
                        self.set_group(idx, &mut group, &mut assigned);
                        if group.spilled() {
                            self.estimate(group.into_vec()) // No reallocation
                        } else {
                            let eval = self
                                .clone()
                                .evaluate(0, &group)
                                .expect("Valid assignment exists");
                            self.brute_force(&group, &eval);
                            self.fast_search()
                                .or_else(|| self.probabilistic_search(&group, &eval))
                        }
                    })
                    .flatten()
            })
            .max();
        let unknowns_unassigned = assigned
            .iter()
            .enumerate()
            .filter_map(|(idx, is_assigned)| {
                (!is_assigned && self.board[idx] == Status::Unknown).then(|| idx)
            })
            .collect::<Vec<Index>>();
        max(best, self.estimate_unassigned(unknowns_unassigned))
    }

    fn search(&mut self) -> Result<(), ScoredUnknown> {
        util::unerr(self.center_search())?;
        util::unerr(self.fast_search())?;
        self.make_consistent_all().unwrap();
        util::unerr(self.fast_search())?;
        util::unerr(self.slow_search())?;
        Ok(())
    }

    pub fn step(&mut self) -> Option<ScoredUnknown> {
        let info = self.search().err();
        if let Some((p, idx)) = info {
            let (row, col) = self.as_rc(idx);
            log::debug!("Guess ({:02}, {:02}): {:.1}%", row, col, p * 100.0);
        }
        info
    }
}
