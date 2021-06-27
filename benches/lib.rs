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

/* Iteration 4a (Backtrack threshold = 16)
Beginner 63.6% (17377 / 27301)
test bench_random_beginner     ... bench:     187,178 ns/iter (+/- 57,746)
Intermediate 42.8% (4241 / 9901)
test bench_random_intermediate ... bench:     427,265 ns/iter (+/- 138,339)
Expert 4.5% (204 / 4501)
test bench_random_expert       ... bench:     798,028 ns/iter (+/- 376,861)
*/

/* Iteration 4b (Backtrack threshold = 32)
Beginner 63.6% (17354 / 27301)
test bench_random_beginner     ... bench:     212,455 ns/iter (+/- 77,739)
Intermediate 47.1% (6364 / 13501)
test bench_random_intermediate ... bench:     600,766 ns/iter (+/- 410,004)
Expert 11.0% (232 / 2101)
test bench_random_expert       ... bench:   2,007,857 ns/iter (+/- 3,321,719)
*/

/* Iteration 5a (Backtrack threshold = 32)
Beginner 74.3% (2231 / 3001)
test bench_random_beginner     ... bench:     761,175 ns/iter (+/- 1,273,958)
Intermediate 57.2% (2231 / 3901)
test bench_random_intermediate ... bench:   2,978,890 ns/iter (+/- 3,990,878)
Expert 19.4% (175 / 901)
test bench_random_expert       ... bench:   8,174,460 ns/iter (+/- 27,425,363)
*/

/* Iteration 5b (Backtrack threshold = 64)
Beginner 74.0% (1998 / 2701)
test bench_random_beginner     ... bench:     753,587 ns/iter (+/- 1,301,422)
Intermediate 57.6% (1556 / 2701)
test bench_random_intermediate ... bench:   2,707,823 ns/iter (+/- 11,399,150)
Expert 21.9% (66 / 301)
test bench_random_expert       ... bench:   7,200,480 ns/iter (+/- 118,105,798)
*/

/* Iteration 6
Beginner 77.0% (2312 / 3001)
test bench_random_beginner     ... bench:     729,070 ns/iter (+/- 1,257,049)
Intermediate 59.8% (2155 / 3601)
test bench_random_intermediate ... bench:   2,183,839 ns/iter (+/- 7,182,664)
Expert 23.0% (207 / 901)
test bench_random_expert       ... bench:   8,335,486 ns/iter (+/- 76,189,221)
*/

// cargo +nightly bench -- --nocapture
fn bench_random(diff: Difficulty, b: &mut Bencher) {
    let mut solved = 0;
    let mut n = 0;
    b.iter(|| {
        let inst = MockMinesweeper::from_difficulty(diff, Some(n));
        match std::panic::catch_unwind(|| Solver::new(inst).solve()) {
            Err(_) => unreachable!("Panicked [Seed {}]", n),
            Ok(res) => match res {
                Err(MinesweeperError::RevealedBomb(_)) => (),
                Err(x) => unreachable!("{} [Seed {}]", x, n),
                Ok(()) => {
                    solved += 1;
                }
            }
        }
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