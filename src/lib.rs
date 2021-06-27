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

mod mock;
pub use mock::MockMinesweeper;

mod solve;
pub use solve::Solver;

mod state;
use state::{MinesweeperState, ShowState};

mod sweep;
use sweep::Minesweeper;

use std::fmt;

use noisy_float::prelude::R64;
type ProbWithIndex = (f64, usize);