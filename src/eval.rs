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

use std::ops::Add;

fn zip_with<T, U, F: Fn(T, T) -> U>(a: Group<T>, b: Group<T>, f: F) -> Group<U> {
    a.into_iter().zip(b.into_iter()).map(|(x, y)| f(x, y)).collect()
}

fn add_with<T: Add>(a: Group<T>, b: Group<T>) -> Group<<T as std::ops::Add>::Output> {
    zip_with(a, b, T::add)
}

pub struct Evaluation {
    pub conf_counts: Group<usize>,
    pub mark_counts: Group<Group<usize>>,
}

impl Add for Evaluation {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Evaluation {
            conf_counts: add_with(self.conf_counts, other.conf_counts),
            mark_counts: zip_with(self.mark_counts, other.mark_counts, add_with),
        }
    }
}
