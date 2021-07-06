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

pub fn binomial(n: usize, r: usize) -> R64 {
    debug_assert!(n >= r);
    (1..=r)
        .map(|x| R64::new(1.0 + ((n - r) as f64) / (x as f64)))
        .product::<R64>()
}

pub fn lift<T, F: Fn(T, T) -> T>(f: F) -> impl Fn(Option<T>, Option<T>) -> Option<T> {
    move |a, b| match (a, b) {
        (Some(x), Some(y)) => Some(f(x, y)),
        (a, b) => a.or(b),
    }
}

pub fn wrap<T>(x: Option<T>) -> Result<(), Option<T>> {
    match x {
        None => Ok(()),
        x => Err(x),
    }
}

pub fn guard<T>(x: bool) -> Result<(), Option<T>> {
    if x {
        Ok(())
    } else {
        Err(None)
    }
}

pub fn guard_from<T, U>(mut f: impl FnMut() -> Option<T>) -> Result<T, Option<U>> {
    match f() {
        None => Err(None),
        Some(x) => Ok(x),
    }
}

pub fn catch<T>(mut f: impl FnMut() -> Result<(), Option<T>>) -> Option<T> {
    f().err().flatten()
}
