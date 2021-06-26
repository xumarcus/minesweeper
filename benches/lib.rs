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

#![feature(test)]

extern crate test;
use test::Bencher;

use minesweeper::*;

/* Iteration 1 (Naive)
running 3 tests
Beginner 72.6% (27444 / 37801)
test bench_random_beginner     ... bench:      88,506 ns/iter (+/- 7,562)
Intermediate 40.0% (1802 / 4501)
test bench_random_intermediate ... bench:     731,141 ns/iter (+/- 346,418)
Expert 1.1% (51 / 4501)
test bench_random_expert       ... bench:     895,050 ns/iter (+/- 513,902)
*/

/* Iteration 2 (Naive)
Beginner 72.9% (55747 / 76501)
test bench_random_beginner     ... bench:      71,006 ns/iter (+/- 5,662)
Intermediate 39.6% (3683 / 9301)
test bench_random_intermediate ... bench:     388,390 ns/iter (+/- 114,617)
Expert 1.4% (127 / 9301)
test bench_random_expert       ... bench:     521,430 ns/iter (+/- 235,622)
*/

/* Iteration 3 (Naive)
Beginner 72.5% (55270 / 76201)
test bench_random_beginner     ... bench:      44,528 ns/iter (+/- 3,481)
Intermediate 40.1% (7813 / 19501)
test bench_random_intermediate ... bench:     191,858 ns/iter (+/- 31,416)
Expert 1.3% (241 / 18901)
test bench_random_expert       ... bench:     284,593 ns/iter (+/- 78,911)
*/

// cargo +nightly bench -- --nocapture
fn bench_random(diff: Difficulty, b: &mut Bencher) {
    let mut solved = 0;
    let mut n = 0;
    b.iter(|| {
        let inst = MockMinesweeper::from_difficulty(diff);
        solved += Solver::new(inst).solve().is_ok() as usize;
        n += 1;
    });
    let percent = 100.0 * (solved as f64) / (n as f64);
    println!("{:?} {:.1}% ({} / {})", diff, percent, solved, n);
}

#[bench]
fn bench_random_beginner(b: &mut Bencher) {
    bench_random(Difficulty::Beginner, b);
}

#[bench]
fn bench_random_intermediate(b: &mut Bencher) {
    bench_random(Difficulty::Intermediate, b);
}

#[bench]
fn bench_random_expert(b: &mut Bencher) {
    bench_random(Difficulty::Expert, b);
}