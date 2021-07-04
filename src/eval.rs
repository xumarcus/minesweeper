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

fn index_join<'a, T, F: 'a + Fn(T, T) -> T>(
    a: Vec<(Index, T)>,
    b: Vec<(Index, T)>,
    f: F,
) -> Vec<(Index, T)> {
    a.into_iter()
        .zip(b.into_iter())
        .map(|((i, x), (j, y))| {
            debug_assert_eq!(i, j);
            (i, f(x, y))
        })
        .collect()
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct PF(Vec<R64>);

impl PF {
    pub fn new(size: usize) -> Self {
        PF(vec![R64::new(0.0); size])
    }

    pub fn one_hot(x: usize) -> Self {
        let mut pf = PF::new(x + 1);
        pf.0[x] = R64::new(1.0);
        pf
    }

    pub fn zip_with<'a, F: 'a + Fn(R64, R64) -> R64>(&self, rhs: &Self, f: F) -> Self {
        PF(self
            .0
            .iter()
            .zip(rhs.0.iter())
            .map(|(&x, &y)| f(x, y))
            .collect())
    }

    #[rustfmt::skip]
    pub fn zip_with_longest<'a, F: 'a + Fn(R64, R64) -> R64>(&self, rhs: &Self, f: F, default: R64) -> Self {
        PF(self.0.iter()
            .zip_longest(rhs.0.iter())
            .map(|either| match either {
                EitherOrBoth::Both(&c, &d) => f(c, d),
                EitherOrBoth::Left(&c)     => f(c, default),
                EitherOrBoth::Right(&d)    => f(default, d),
            })
            .collect())
    }

    pub fn convolve(&self, rhs: &Self) -> Self {
        let mut pf = PF::new(self.0.len() + rhs.0.len() + 1);
        for (i, &x) in self.0.iter().enumerate() {
            for (j, &y) in rhs.0.iter().enumerate() {
                pf.0[i + j] += x * y;
            }
        }
        pf
    }
}

impl Add for PF {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        self.zip_with_longest(&rhs, |x, y| x + y, R64::new(0.0))
    }
}

impl Add for &PF {
    type Output = PF;
    fn add(self, rhs: Self) -> PF {
        self.zip_with_longest(rhs, |x, y| x + y, R64::new(0.0))
    }
}

impl Mul for PF {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self {
        self.zip_with(&rhs, |x, y| x * y)
    }
}

impl Mul for &PF {
    type Output = PF;
    fn mul(self, rhs: Self) -> PF {
        self.zip_with(rhs, |x, y| x * y)
    }
}

#[derive(Clone, Debug)]
pub struct Evaluation {
    count: R64,
    spf: PF,
    ipf: Vec<(Index, PF)>,  // TODO one-indexed
}

impl Evaluation {
    pub fn new(state: &MinesweeperState, idx: Index) -> Self {
        let count = R64::new(1.0);
        let (spf, ipf) = match state.get(idx) {
            Status::Flagged => (PF::one_hot(1), vec![(idx, PF::one_hot(1))]),
            Status::Marked => (PF::one_hot(0), vec![(idx, PF::new(1))]),
            _ => unreachable!(),
        };
        Self { count, spf, ipf }
    }

    pub fn label_certains(&self, state: &mut MinesweeperState) {
        for (idx, pf) in &self.ipf {
            debug_assert!(!pf.0.is_empty()); // TODO one-indexed
            if let Some(s) = pf.0.get(1..) {
                if !s.is_empty() && s.iter().all(|&p| p.raw() == 1.0) {
                    state.set_flag(*idx);
                }
            }
            if pf.0.iter().all(|&p| p.raw() == 0.0) {
                state.set_mark(*idx);
            }
        }
    }
}

impl Add for Evaluation {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        let count = self.count + rhs.count;
        log::debug!("{}", count);
        let p = self.count / count;
        let q = rhs.count / count;
        let f = |c: R64, d: R64| c * p + d * q;
        let g = |x: PF, y: PF| x.zip_with_longest(&y, f, R64::new(0.0));
        Self {
            count,
            spf: g(self.spf, rhs.spf),
            ipf: index_join(self.ipf, rhs.ipf, g),
        }
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
