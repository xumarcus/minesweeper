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

use std::cmp::{max, min};

use rand::Rng;

#[derive(Clone, Debug)]
pub struct Config {
    width: usize,
    length: usize,
    mines: usize,
    seed: u64,
}

impl Config {
    pub fn new(width: usize, length: usize, mines: usize, seed: Option<u64>) -> MsResult<Self> {
        if width * length < mines {
            return Err(MinesweeperError::NumberOfMinesOutOfRange);
        }
        let seed = seed.unwrap_or_else(rand::random::<u64>);
        log::debug!("Create seed {}", seed);
        let mut inst = Self { width, length, mines, squares: vec![], seed };
        inst.squares = (0..inst.size())
            .map(|idx| {
                let (row, col) = inst.as_rc(idx);
                let rmin = max(1, row) - 1;
                let rmax = min(inst.width() - 1, row + 1);
                let cmin = max(1, col) - 1;
                let cmax = min(inst.length() - 1, col + 1);
                let mut av = Square::new();
                for r in rmin..=rmax {
                    for c in cmin..=cmax {
                        let cidx = inst.from_rc(r, c);
                        if cidx != idx {
                            av.push(cidx);
                        }
                    }
                }
                av
            })
            .collect();
        inst
        })
        .ok_or()
    }

    #[inline]
    pub fn width(&self) -> usize {
        self.width
    }

    #[inline]
    pub fn length(&self) -> usize {
        self.length
    }

    #[inline]
    pub fn size(&self) -> usize {
        self.width * self.length
    }

    #[inline]
    pub fn mines(&self) -> usize {
        self.mines
    }

    #[inline]
    pub fn as_rc(&self, idx: Index) -> (usize, usize) {
        (idx / self.length, idx % self.length)
    }

    #[inline]
    pub fn from_rc(&self, row: usize, col: usize) -> Index {
        row * self.length + col
    }

    #[inline]
    pub fn center(&self) -> Index {
        self.from_rc(self.width() / 2, self.length() / 2)
    }

    #[inline]
    pub fn new_rng(&self) -> impl Rng {
        StdRng::seed_from_u64(self.seed)
    }

    #[inline]
    pub fn random_index(&self, rng: &mut impl Rng) -> usize {
        let w = Uniform::from(0..self.width());
        let l = Uniform::from(0..self.length());
        self.from_rc(w.sample(rng), l.sample(rng))
    }

    #[inline]
    pub fn square(&self, idx: Index) -> &[Index] {
        &self.squares[idx]
    }
    
    #[rustfmt::skip]
    pub fn from_difficulty(diff: Difficulty, seed: Option<u64>) -> Self {
        let result = match diff {
            Difficulty::Beginner     => Self::new( 9,  9, 10, seed),
            Difficulty::Intermediate => Self::new(16, 16, 40, seed),
            Difficulty::Expert       => Self::new(16, 30, 99, seed),
        };
        result.unwrap()
    }
}