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

pub struct Evaluation {
    count: R64,
    p_flags_eq: Vec<R64>,
    p_is_flag_given_total: Vec<(Index, Vec<R64>)>,
}

impl Evaluation {
    pub fn new(state: &MinesweeperState, group: &BitVec) -> Self {
        let group_flags = group.iter_ones().filter(|&idx| state.get(idx) == Status::Flagged).count();
        Self {
            count: R64::new(1.0),
            p_flags_eq: one_hot(group_flags),
            p_is_flag_given_total: group.iter_ones().map(|idx| (idx, one_hot(group_flags))).collect()
        }
    }

    /*
    pub fn add_same_group_evals(a: Self, b: Self) -> Self {
        let count = a.count + b.count;
        let merge_prob = |c: R64, d: R64| (c * a.count + d * b.count) / count;
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
    */

    pub fn add_disj_group_evals(unknowns_remaining: usize, flags_remaining: usize, a: Self, b: Self) -> Self {
        let count = a.count * b.count;
        let len = a.p_flags_eq.len() + b.p_flags_eq.len() + 1;
        let mut p_flags_eq = vec![R64::new(0.0); len]; 
        for (i, x) in a.p_flags_eq.into_iter().enumerate() {
            for (j, y) in b.p_flags_eq.into_iter().enumerate() {
                p_flags_eq[i + j] += x * y;
            }
        }
        let new_ifgt = |other: &Self, ifgt: &[(Index, Vec<R64>)]| ifgt
            .iter()
            .map(|(idx, orig)| {
                let max_flags = orig.len() + other.p_flags_eq.len() - 2;
                let weights = (0..orig.len())
                    .map(|i| {
                        let n = ifgt.len();
                        let n_bar = unknowns_remaining - other.p_is_flag_given_total.len();
                        let numer = flags_remaining - max_flags + i;
                        let p = (numer as f64) / (n_bar as f64);
                        debug_assert!(0.0 <= p && p <= 1.0);
                        R64::new(Binomial::new(n, p).mass(i))
                    })
                    .collect::<Vec<R64>>();
                let w_sum = weights.iter().sum::<R64>();
                let new_vec = (0..=max_flags)
                    .map(|flag_count| orig
                        .into_iter()
                        .zip(weights.into_iter())
                        .map(|(&p, w)| p * w / w_sum)
                        .sum::<R64>()
                    )
                    .collect();
                (*idx, new_vec)
            });
        let p_is_flag_given_total = new_ifgt(&b, &a.p_is_flag_given_total)
            .merge(new_ifgt(&a, &b.p_is_flag_given_total))
            .collect();
        Self {
            count,
            p_flags_eq,
            p_is_flag_given_total
        }
    }
}