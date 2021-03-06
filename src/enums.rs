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

#[derive(Clone, Copy, Debug, Error)]
pub enum MinesweeperError {
    #[error("NumberOfMinesOutOfRange")]
    NumberOfMinesOutOfRange,
    #[error("RevealedBomb")]
    RevealedBomb(usize),
}

pub type MsResult<T> = Result<T, MinesweeperError>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Status {
    Flagged,
    Known(usize),
    Marked,
    Unknown,
}

impl Default for Status {
    fn default() -> Self {
        Self::Unknown
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, EnumString)]
pub enum Difficulty {
    Beginner,
    Intermediate,
    Expert,
}

impl Default for Difficulty {
    fn default() -> Self {
        Self::Beginner
    }
}
