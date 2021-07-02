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

use itertools::{EitherOrBoth, Itertools};

fn one_hot(x: usize) -> Vec<R64> {
    let mut v = vec![R64::new(0.0); x + 1];
    v[x] = R64::new(1.0);
    v
}

#[derive(Clone, Debug)]
pub struct Evaluation {
    count: R64,
    p_flags_eq: Vec<R64>,
    p_is_flag_given_total: Vec<(Index, Vec<R64>)>,
}

impl Evaluation {
    pub fn new(state: &MinesweeperState, idx: Index) -> Self {
        let p_flags_eq = match state.get(idx) {
            Status::Flagged => vec![R64::new(0.0), R64::new(1.0)],
            Status::Marked => vec![R64::new(1.0)],
            _ => unreachable!(),
        };
        let p_is_flag_given_total = vec![(idx, p_flags_eq.clone())];
        Self {
            count: R64::new(1.0),
            p_flags_eq,
            p_is_flag_given_total
        }
    }

    fn ifgt_of_other(&self, unknowns_remaining: usize, flags_remaining: usize, ifgt: &[(Index, Vec<R64>)]) -> Vec<(Index, Vec<R64>)> {
        ifgt.iter().map(|(idx, orig)| {
            let n = ifgt.len();
            let denom = unknowns_remaining - self.p_is_flag_given_total.len();
            let max_flags = orig.len() + self.p_flags_eq.len() - 2;
            let new_vec = (0..=max_flags)
                .map(|flag_count| {
                    let weights = (0..orig.len())
                        .map(|i| {
                            let numer = flags_remaining - flag_count + i;
                            let p = (numer as f64) / (denom as f64);
                            debug_assert!(0.0 <= p && p <= 1.0);
                            R64::new(Binomial::new(n, p).mass(i))
                        })
                        .collect::<Vec<R64>>();
                    let w_sum = weights.iter().sum::<R64>();
                    orig
                        .into_iter()
                        .zip(weights)
                        .map(|(&p, w)| p * w / w_sum)
                        .sum::<R64>()
                })
                .collect();
            (*idx, new_vec)
        })
        .collect()
    }

    pub fn add_same_group_evals(a: Self, b: Self) -> Self {
        let count = a.count + b.count;
        let a_p = a.count / count;
        let b_p = b.count / count;
        let merge_prob = |c: R64, d: R64| c * a_p + d * b_p;
        let p_flags_eq = a.p_flags_eq.into_iter()
            .zip_longest(b.p_flags_eq.into_iter())
            .map(|either| match either {
                EitherOrBoth::Both(x, y) => x + y,
                EitherOrBoth::Left(x) | EitherOrBoth::Right(x) => x,
            })
            .collect();
        let p_is_flag_given_total = a.p_is_flag_given_total.into_iter()
            .zip(b.p_is_flag_given_total.into_iter())
            .map(|((i, x), (j, y))| {
                debug_assert_eq!(i, j);
                let z = x.into_iter()
                    .zip_longest(y.into_iter())
                    .map(|either| match either {
                        EitherOrBoth::Both(c, d) => merge_prob(c, d),
                        EitherOrBoth::Left(c) => merge_prob(c, R64::new(0.0)),
                        EitherOrBoth::Right(d) => merge_prob(R64::new(0.0), d),
                    })
                    .collect();
                (i, z)
            })
            .collect();
        Self {
            count,
            p_flags_eq,
            p_is_flag_given_total
        }
    }

    pub fn add_disj_group_evals(unknowns_remaining: usize, flags_remaining: usize, a: Self, b: Self) -> Self {
        let count = a.count * b.count;
        let len = a.p_flags_eq.len() + b.p_flags_eq.len() + 1;
        let mut p_flags_eq = vec![R64::new(0.0); len]; 
        for (i, &x) in a.p_flags_eq.iter().enumerate() {
            for (j, &y) in b.p_flags_eq.iter().enumerate() {
                p_flags_eq[i + j] += x * y; 
            }
        }
        let p_is_flag_given_total = b.ifgt_of_other(unknowns_remaining, flags_remaining, &a.p_is_flag_given_total).into_iter()
            .merge(a.ifgt_of_other(unknowns_remaining, flags_remaining, &b.p_is_flag_given_total).into_iter())
            .collect();
        Self {
            count,
            p_flags_eq,
            p_is_flag_given_total
        }
    }

    pub fn label_certains(&self, state: &mut MinesweeperState) {
        for (idx, pv) in &self.p_is_flag_given_total {
            debug_assert!(!pv.is_empty());
            if pv.iter().all(|&p| p.raw() == 1.0) {
                state.set_flag(*idx);
            }
            if pv.iter().all(|&p| p.raw() == 0.0) {
                state.set_mark(*idx);
            }
        }
    }
}