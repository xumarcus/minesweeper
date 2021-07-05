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

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
struct PF(Vec<R64>);

impl fmt::Debug for PF {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[")?;
        for (i, p) in self.0.iter().enumerate() {
            if p > &R64::new(0.0) {
                write!(f, "{:02}: {:.3}; ", i, p.raw())?;
            }
        }
        write!(f, "]")?;
        Ok(())
    }
}

impl PF {
    fn new(size: usize) -> Self {
        PF(vec![R64::new(0.0); size])
    }

    fn one_hot(x: usize) -> Self {
        let mut pf = PF::new(x + 1);
        pf.0[x] = R64::new(1.0);
        pf
    }

    fn zip_with_longest<'a, F: 'a + Fn(R64, R64) -> R64>(
        &self,
        rhs: &Self,
        f: F,
        default: R64,
    ) -> Self {
        PF(self
            .0
            .iter()
            .zip_longest(rhs.0.iter())
            .map(|either| match either {
                EitherOrBoth::Both(&c, &d) => f(c, d),
                EitherOrBoth::Left(&c) => f(c, default),
                EitherOrBoth::Right(&d) => f(default, d),
            })
            .collect())
    }

    fn convolve(&self, rhs: &Self) -> Self {
        let mut pf = PF::new(self.0.len() + rhs.0.len() + 1);
        for (i, &x) in self.0.iter().enumerate() {
            for (j, &y) in rhs.0.iter().enumerate() {
                pf.0[i + j] += x * y;
            }
        }
        pf
    }

    pub fn ev(&self) -> R64 {
        self.0
            .iter()
            .enumerate()
            .map(|(idx, &p)| p * R64::new(idx as f64))
            .sum::<R64>()
    }

    // Precision?
    fn normalize(&mut self) {
        let s = self.0.iter().sum::<R64>();
        if s > R64::new(0.0) {
            for p in self.0.iter_mut() {
                *p *= R64::new(1.0) / s;
            }
        }
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
        self.zip_with_longest(&rhs, |x, y| x * y, R64::new(0.0))
    }
}

impl Mul for &PF {
    type Output = PF;
    fn mul(self, rhs: Self) -> PF {
        self.zip_with_longest(rhs, |x, y| x * y, R64::new(0.0))
    }
}

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
                    Status::Marked => PF::new(1), // @todo one-indexed
                    _ => unreachable!(),
                };
                (idx, pf)
            })
            .collect();
        Self { count, spf, ipf }
    }
    
    pub fn label(&self, state: &mut MinesweeperState) {
        for (idx, pf) in &self.ipf {
            debug_assert!(!pf.0.is_empty()); // TODO one-indexed
            if let Some(s) = pf.0.get(1..) {
                if !s.is_empty() && s.iter().all(|&p| p == R64::new(1.0)) {
                    state.set_flag(*idx);
                }
            }
            if pf.0.iter().all(|&p| p == R64::new(0.0)) {
                state.set_mark(*idx);
            }
        }
    }

    pub fn search(&self, flags: usize, n: usize, idx: Option<Index>) -> Option<ScoredIndex> {
        let iter = self.spf.0.iter().enumerate();
        let mut cpf = PF(iter.filter_map(|(i, p)|  (i <= flags && flags <= n + i).then(|| util::binomial(n, flags - i) * p)).collect());
        cpf.normalize();
        log::debug!("CPF {:?}", cpf);
        self.ipf
            .iter()
            .map(|(idx, pf)| ((&cpf * pf).0.iter().sum::<R64>(), *idx))
            .inspect(|(p, idx)| log::debug!("{:03} {:.3}", idx, p))
            .chain(idx
                .map(|idx| ((R64::new(flags as f64) - *&cpf.ev()) / R64::new(n as f64), idx))
                .into_iter()
                .inspect(|(p, _)| log::debug!("Base [{:.3}]", p))
            )
            .min()
    }
}

impl Add for Evaluation {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        let count = self.count + rhs.count;
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
