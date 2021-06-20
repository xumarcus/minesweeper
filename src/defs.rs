use super::*;

pub struct Config {
    width: usize,
    length: usize,
    mines: usize,
}

pub struct MinesweeperInstance<T: Cell> {
    board: Vec<T>,
    config: Config,
}

pub trait Minesweeper {
    fn reveal(&mut self, idx: usize) -> MsResult<()>;
    fn get_cells<T: Cell>(&self) -> &Vec<T>;
    fn get_cells_mut<T: Cell>(&mut self) -> &mut Vec<T>;
    fn get_config(&self) -> &Config;
    fn solve_next(&mut self) -> MsResult<(usize, NotNan<f64>)>;
    fn solve(&mut self) -> MsResult<()>;
}