mod config;
pub use config::Config;

mod enums;
pub use enums::*;

mod mock;
pub use mock::MockMinesweeper;

mod solve;
pub use solve::*;

use std::fmt;
