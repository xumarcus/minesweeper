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

pub fn lift<T, F: Fn(T, T) -> T>(a: Option<T>, b: Option<T>, f: F) -> Option<T> {
    match (a, b) {
        (Some(x), Some(y)) => Some(f(x, y)),
        (a, b) => a.or(b),
    }
}

pub fn one_hot(size: usize, idx: usize) -> Group<usize> {
    let mut v = smallvec![0; size];
    v[idx] = 1;
    v
}

pub fn unerr<T: Default, E>(x: Option<E>) -> Result<T, E> {
    match x {
        Some(x) => Err(x),
        None => Ok(T::default()),
    }
}
