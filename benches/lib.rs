#![feature(test)]
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

extern crate test;
use test::Bencher;

use minesweeper::{Difficulty, Minesweeper};

/* Iteration 1 (Naive)
running 3 tests
Beginner 72.6% (27444 / 37801)
test bench_random_beginner     ... bench:      88,506 ns/iter (+/- 7,562)
Intermediate 40.0% (1802 / 4501)
test bench_random_intermediate ... bench:     731,141 ns/iter (+/- 346,418)
Expert 1.1% (51 / 4501)
test bench_random_expert       ... bench:     895,050 ns/iter (+/- 513,902)
*/

// cargo +nightly  bench -- --nocapture
fn bench_random(diff: &Difficulty, b: &mut Bencher) {
    let mut solved = 0;
    let mut n = 0;
    b.iter(|| {
        let mut inst = Minesweeper::from_difficulty(&diff);
        if inst.solve().is_ok() {
            solved += 1;
        }
        n += 1;
    });
    println!(
        "{:?} {:.1}% ({} / {})",
        diff,
        100.0 * (solved as f64) / (n as f64),
        solved,
        n
    );
}

#[bench]
fn bench_random_beginner(b: &mut Bencher) {
    bench_random(&Difficulty::Beginner, b);
}

#[bench]
fn bench_random_intermediate(b: &mut Bencher) {
    bench_random(&Difficulty::Intermediate, b);
}

#[bench]
fn bench_random_expert(b: &mut Bencher) {
    bench_random(&Difficulty::Expert, b);
}
