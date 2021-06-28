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

mod enums;
pub use enums::*;

mod eval;
use eval::Evaluation;

mod mock;
pub use mock::MockMinesweeper;

mod solve;
pub use solve::Solver;

mod showstate;
use showstate::ShowState;

mod state;
use state::MinesweeperState;

mod sweep;
use sweep::Minesweeper;

mod util;

use noisy_float::prelude::R64;

use smallvec::{smallvec, SmallVec};

const GROUP_SIZE: usize = 16;

type Index = usize;
type Group<T> = SmallVec<[T; GROUP_SIZE]>;
type ScoredUnknown = (R64, usize);
type Square = SmallVec<[Index; 8]>;