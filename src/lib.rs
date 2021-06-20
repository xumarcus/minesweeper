mod config;
pub use config::Config;

mod enums;
pub use enums::*;

mod mock;
pub use mock::MockMinesweeper;

use ordered_float::NotNan;
use std::fmt;

// TODO separate
pub trait Minesweeper {
    // Cuz solve_next is generic
    fn get_config(&self) -> Config;
    fn get_cells(&self) -> &Vec<Status>;
    fn get_cells_mut(&mut self) -> &mut Vec<Status>;

    // Implementation differs
    fn reveal(&mut self, idx: usize) -> MsResult<()>;
    
    // Generic
    //fn solve_next(&mut self) -> MsResult<(usize, NotNan<f64>)>;
    //fn solve(&mut self) -> MsResult<()>;
}