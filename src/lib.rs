mod cells;
use cells::*;

mod config;
pub use config::Config;

mod enums;
pub use enums::*;

mod mock;
pub use mock::MockMinesweeper;

use ordered_float::NotNan;
use std::fmt;

pub trait Minesweeper {
    type Item: Cell;

    fn get_config(&self) -> Config;

    // Implementation differs for MockMinesweeper
    // | RealMinesweeper (Hook process) | InteractiveMinesweeper (Ask for input)

    fn reveal(&mut self, idx: usize) -> MsResult<()>;
    fn get_cells(&self) -> &Vec<Self::Item>;
    fn get_cells_mut(&mut self) -> &mut Vec<Self::Item>;
    //fn solve_next(&mut self) -> MsResult<(usize, NotNan<f64>)>;
    //fn solve(&mut self) -> MsResult<()>;
}

// Workaround
pub struct Show<T>(pub T);

impl<T: Minesweeper<Item = impl fmt::Display>> fmt::Display for Show<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let config = self.0.get_config();
        writeln!(f, "Dimensions: {} x {}", config.width, config.length)?;
        writeln!(f, "Flagged: {} / {}", 0, config.mines)?;
        let board = self.0.get_cells();
        for row in 0..config.width {
            let idx = row * config.length;
            if let Some(slice) = board.get(idx..idx + config.length) {
                for cell in slice {
                    write!(f, "{} ", cell)?;
                }
            }
            write!(f, "\n")?;
        }
        Ok(())
    }
}