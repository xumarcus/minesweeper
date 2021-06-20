#![feature(test)]
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
