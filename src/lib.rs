mod cells;
use cells::*;

mod defs;
pub use defs::*;

mod enums;
use enums::*;

use ordered_float::NotNan;

pub use enums::{Difficulty, MinesweeperError};