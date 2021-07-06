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

#[derive(Clone)]
pub struct Evaluation {
    count: R64,
    spf: PF,
    ipf: Vec<(Index, PF)>, // @todo one-indexed
}

impl fmt::Debug for Evaluation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Count [{}]", self.count)?;
        writeln!(f, "SPF {:?}", self.spf)?;
        for (idx, pf) in &self.ipf {
            writeln!(f, "{:03} {:?}", idx, pf)?;
        }
        Ok(())
    }
}

impl Evaluation {
    pub fn new(state: &MinesweeperState, remainder: BitVec) -> Self {
        let count = R64::new(1.0);
        let flags = remainder
            .iter_ones()
            .filter(|&idx| matches!(state.get(idx), Status::Flagged))
            .count();
        let spf = PF::one_hot(flags);
        let ipf = remainder
            .iter_ones()
            .map(|idx| {
                let pf = match state.get(idx) {
                    Status::Flagged => PF::one_hot(flags),
                    Status::Marked => PF::default(), // @todo one-indexed
                    _ => unreachable!(),
                };
                (idx, pf)
            })
            .collect();
        Self { count, spf, ipf }
    }

    pub fn label(&self, state: &mut MinesweeperState) -> bool {
        self.ipf.iter().all(|(idx, pf)| pf.label(state, *idx))
    }

    pub fn to_probabilities(
        &self,
        flags: usize,
        n: usize,
    ) -> (Option<R64>, impl Iterator<Item = ScoredIndex> + '_) {
        let mut weighted_spf = self.spf.clone();
        weighted_spf.weighted(flags, n);
        let bp =
            (n != 0).then(|| (R64::new(flags as f64) - *&weighted_spf.ev()) / R64::new(n as f64));
        let ps = self.ipf.iter().map(move |(idx, pf)| {
            let p = (&weighted_spf * pf).sum();
            (p, *idx)
        });
        (bp, ps)
    }
}

impl Add for Evaluation {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        let count = self.count + rhs.count;
        let p = self.count / count;
        let q = rhs.count / count;
        let g =
            |x: PF, y: PF| x.zip_with_longest(&y, |c: R64, d: R64| c * p + d * q, R64::new(0.0));
        let spf = g(self.spf, rhs.spf);
        let ipf = self
            .ipf
            .into_iter()
            .zip(rhs.ipf.into_iter())
            .map(|((i, x), (j, y))| {
                debug_assert_eq!(i, j);
                (i, g(x, y))
            })
            .collect();
        Self { count, spf, ipf }
    }
}

impl Mul for Evaluation {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self {
        let count = self.count * rhs.count;
        let spf = self.spf.convolve(&rhs.spf);
        let lhs_ipf = self
            .ipf
            .iter()
            .map(|(idx, pf)| (*idx, (pf * &self.spf).convolve(&rhs.spf)));
        let rhs_ipf = rhs
            .ipf
            .iter()
            .map(|(idx, pf)| (*idx, (pf * &rhs.spf).convolve(&self.spf)));
        let ipf = lhs_ipf.merge(rhs_ipf).collect();
        Self { count, spf, ipf }
    }
}
