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

mod config;
pub use config::Config;

mod enums;
pub use enums::*;

mod eval;
use eval::Evaluation;

mod group;
use group::Group;

mod interface;
pub use interface::Minesweeper;

mod mock;
pub use mock::MockMinesweeper;

mod pf;
use pf::PF;

mod show;
pub use show::ShowMinesweeper;

mod solve;
pub use solve::Solver;

mod state;
use state::MinesweeperState;

mod util;

use arrayvec::ArrayVec;
use bitvec::prelude::*;
use itertools::{EitherOrBoth, Itertools};
use noisy_float::prelude::R64;
use rand::{
    self,
    distributions::{Distribution, Uniform},
    rngs::StdRng,
    Rng, SeedableRng,
};
use std::cmp::{max, min};
use std::fmt;
use std::ops::{Add, Mul};
use strum_macros::EnumString;
use thiserror::Error;

type Index = usize;
type ScoredIndex = (R64, usize);
type Square = ArrayVec<usize, 8>;
