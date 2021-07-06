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

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct PF(Vec<R64>);

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
    pub fn one_hot(x: usize) -> Self {
        let mut v = vec![R64::new(0.0); x + 1];
        v[x] = R64::new(1.0);
        PF(v)
    }

    pub fn zip_with_longest<'a, F: 'a + Fn(R64, R64) -> R64>(
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

    pub fn convolve(&self, rhs: &Self) -> Self {
        let mut v = vec![R64::new(0.0); self.0.len() + rhs.0.len() + 1];
        for (i, &x) in self.0.iter().enumerate() {
            for (j, &y) in rhs.0.iter().enumerate() {
                v[i + j] += x * y;
            }
        }
        PF(v)
    }

    pub fn ev(&self) -> R64 {
        self.0
            .iter()
            .enumerate()
            .map(|(idx, &p)| p * R64::new(idx as f64))
            .sum::<R64>()
    }

    // Precision?
    pub fn normalize(&mut self) {
        let s = self.0.iter().sum::<R64>();
        if s > R64::new(0.0) {
            for p in self.0.iter_mut() {
                *p *= R64::new(1.0) / s;
            }
        }
    }

    pub fn label(&self, state: &mut MinesweeperState, idx: usize) -> bool {
        if self.0.len() > 1 && self.0.iter().skip(1).all(|&p| p == R64::new(1.0)) {
            return state.set_flag(idx);
        }
        if self.0.iter().all(|&p| p == R64::new(0.0)) {
            return state.set_mark(idx);
        }
        true
    }

    pub fn weighted(&mut self, flags: usize, n: usize) {
        for (i, p) in self.0.iter_mut().enumerate() {
            if i <= flags && flags <= n + i {
                *p *= util::binomial(n, flags - i);
            } else {
                *p = R64::new(0.0);
            }
        }
        self.normalize();
    }

    pub fn sum(&self) -> R64 {
        self.0.iter().sum::<R64>()
    }
}

impl Default for PF {
    fn default() -> Self {
        PF(vec![R64::new(0.0)])
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
