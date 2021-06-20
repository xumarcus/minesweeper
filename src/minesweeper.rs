use ordered_float::NotNan;
use rand::{
    self,
    distributions::{Distribution, Uniform},
};
use std::cmp::{max, min};
use std::fmt;

mod cells;
use cells::*;

mod enums;
use enums::*;

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

impl<T: Cell> Minesweeper for MinesweeperInstance<T> {
    fn reveal(&mut self, idx: usize) -> MsResult<()> {

    }
}




#[derive(Clone, Debug)]
pub struct Minesweeper<T: Cell> {
    board: Vec<T>,
    
}

impl<T: Cell + fmt::Display> fmt::Display for Minesweeper<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Dimensions: {} x {}", self.width, self.length)?;
        writeln!(
            f,
            "Flagged: {} / {}",
            self.count_status(Status::Flagged),
            self.mines
        )?;
        for row in 0..self.width {
            let idx = row * self.length;
            if let Some(slice) = self.board.get(idx..idx + self.length) {
                for cell in slice {
                    write!(f, "{} ", cell)?;
                }
            }
            write!(f, "\n")?;
        }
        Ok(())
    }
}

impl<T: Cell> Minesweeper<T> {
    fn square(&self, idx: usize) -> impl Iterator<Item = usize> {
        let (row, col) = self.as_rc(idx);
        let len = self.length; // Copy length from self
        (max(1, row) - 1..=min(self.width - 1, row + 1))
            .flat_map(move |r| (max(1, col) - 1..=min(len - 1, col + 1)).map(move |c| r * len + c))
    }

    fn count_status(&self, status: Status) -> usize {
        self.board
            .iter()
            .filter(|cell| cell.get_status() == status)
            .count()
    }

    fn count_status_square(&self, idx: usize, status: Status) -> usize {
        self.square(idx)
            .filter(|cidx| self.board[*cidx].get_status() == status)
            .count()
    }

    #[inline]
    pub fn as_rc(&self, idx: usize) -> (usize, usize) {
        (idx / self.length, idx % self.length)
    }

    #[inline]
    pub fn from_rc(&self, row: usize, col: usize) -> usize {
        row * self.length + col
    }


    // 1.0f64 is exact
    pub fn solve_next(&mut self) -> Result<(usize, NotNan<f64>), MinesweeperError> {
        let n = self.board.len();
        let center = self.from_rc(self.width / 2, self.length / 2);
        if self.board[center].status != Status::Visible {
            return Ok((center, NotNan::new(1.0).unwrap()));
        }
        loop {
            let mut to_solve: Vec<Option<bool>> = vec![None; n];
            for (idx, cell) in self.board.iter().enumerate() {
                if let Some(count) = cell.get() {
                    let unknowns = self.count_status_square(idx, Status::Unknown);
                    let flags = self.count_status_square(idx, Status::Flagged);
                    if count == flags {
                        for cidx in self.square(idx) {
                            if self.board[cidx].status == Status::Unknown {
                                to_solve[cidx] = Some(true);
                            }
                        }
                    } else if count == unknowns + flags {
                        for cidx in self.square(idx) {
                            if self.board[cidx].status == Status::Unknown {
                                to_solve[cidx] = Some(false);
                            }
                        }
                    }
                }
            }
            if to_solve.iter().all(|to_s| to_s.is_none()) {
                break;
            }
            for (idx, to_s) in to_solve.iter().enumerate() {
                match to_s {
                    Some(/* to solve */ true) => self.board[idx].status = Status::ToSolve,
                    Some(/* to flag */ false) => self.board[idx].status = Status::Flagged,
                    None => continue,
                }
            }
        }
        let unflagged = self.mines() - self.count_status(Status::Flagged);
        let unknowns = self.count_status(Status::Unknown);
        self.board
            .iter()
            .enumerate()
            .find(|(_, cell)| cell.status == Status::ToSolve)
            .map(|(idx, _)| (idx, NotNan::new(1.0).unwrap()))
            .or_else(|| {
                let base_prob = NotNan::new((unflagged as f64) / (unknowns as f64)).ok()?;
                let mut prob = vec![None; n];
                for (idx, cell) in self.board.iter().enumerate() {
                    if let Some(count) = cell.get() {
                        let cell_p = NotNan::new(
                            (count as f64)
                                / (self.count_status_square(idx, Status::Unknown) as f64),
                        );
                        for idx_sq in self.square(idx) {
                            prob[idx_sq] = max(prob[idx_sq], cell_p.ok());
                        }
                    }
                }
                self.board
                    .iter()
                    .enumerate()
                    .filter(|(_, cell)| cell.status == Status::Unknown)
                    .map(|(idx, _)| (idx, prob[idx].unwrap_or(base_prob)))
                    .min_by_key(|(_, p)| *p)
            })
            .ok_or(MinesweeperError::IsAlreadySolved)
    }

    pub fn solve(&mut self) -> Result<(), MinesweeperError> {
        while let Ok((idx, _)) = self.solve_next() {
            self.reveal(idx)?;
        }
        Ok(())
    }
}

impl Minesweeper<MockCell> {
    pub fn new(width: usize, length: usize, mines: usize) -> Result<Self, MinesweeperError> {
        if width * length <= mines {
            return Err(MinesweeperError::InvalidParameters);
        }
        let mut is_bomb_v = vec![false; width * length];
        let mut inst = Self { board: Vec::new(), width, length, mines };
        let mut rng = rand::thread_rng();
        let w_gen = Uniform::from(0..width);
        let l_gen = Uniform::from(0..length);
        for _ in 0..mines {
            loop {
                let row = w_gen.sample(&mut rng);
                let col = l_gen.sample(&mut rng);
                if row == width / 2 && col == length / 2 {
                    continue; // keep center revealable
                }
                let idx = inst.from_rc(row, col);
                if !is_bomb_v[idx] {
                    is_bomb_v[idx] = true;
                    break;
                }
            }
        }
        inst.board = is_bomb_v.iter().map(MockCell::new).collect();
        Ok(inst)
    }

    #[rustfmt::skip]
    pub fn from_difficulty(diff: &Difficulty) -> Self {
        let result = match diff {
            Difficulty::Beginner     => Self::new( 9,  9, 10),
            Difficulty::Intermediate => Self::new(16, 16, 40),
            Difficulty::Expert       => Self::new(30, 16, 99),
        };
        result.unwrap()
    }

}

#[cfg(test)]
mod tests {
    use super::{Status::*, *};

    fn get_inst() -> Minesweeper<MockCell> {
        Minesweeper {
        board: vec![
            MockCell {
                count: None,
                status: Unknown,
            },
            MockCell {
                count: Some(2),
                status: Unknown,
            },
            MockCell {
                count: Some(1),
                status: Unknown,
            },
            MockCell {
                count: None,
                status: Unknown,
            },
            MockCell {
                count: Some(1),
                status: Unknown,
            },
            MockCell {
                count: Some(0),
                status: Unknown,
            },
            MockCell {
                count: Some(0),
                status: Unknown,
            },
            MockCell {
                count: Some(0),
                status: Unknown,
            },
            MockCell {
                count: Some(0),
                status: Unknown,
            },
            MockCell {
                count: None,
                status: Unknown,
            },
            MockCell {
                count: Some(3),
                status: Unknown,
            },
            MockCell {
                count: Some(1),
                status: Unknown,
            },
            MockCell {
                count: Some(1),
                status: Unknown,
            },
            MockCell {
                count: Some(1),
                status: Unknown,
            },
            MockCell {
                count: Some(1),
                status: Unknown,
            },
            MockCell {
                count: Some(1),
                status: Unknown,
            },
            MockCell {
                count: Some(1),
                status: Unknown,
            },
            MockCell {
                count: Some(0),
                status: Unknown,
            },
            MockCell {
                count: None,
                status: Unknown,
            },
            MockCell {
                count: Some(2),
                status: Unknown,
            },
            MockCell {
                count: Some(0),
                status: Unknown,
            },
            MockCell {
                count: Some(0),
                status: Unknown,
            },
            MockCell {
                count: Some(0),
                status: Unknown,
            },
            MockCell {
                count: Some(1),
                status: Unknown,
            },
            MockCell {
                count: None,
                status: Unknown,
            },
            MockCell {
                count: Some(2),
                status: Unknown,
            },
            MockCell {
                count: Some(1),
                status: Unknown,
            },
            MockCell {
                count: Some(1),
                status: Unknown,
            },
            MockCell {
                count: Some(1),
                status: Unknown,
            },
            MockCell {
                count: Some(0),
                status: Unknown,
            },
            MockCell {
                count: Some(0),
                status: Unknown,
            },
            MockCell {
                count: Some(0),
                status: Unknown,
            },
            MockCell {
                count: Some(1),
                status: Unknown,
            },
            MockCell {
                count: Some(3),
                status: Unknown,
            },
            MockCell {
                count: None,
                status: Unknown,
            },
            MockCell {
                count: Some(2),
                status: Unknown,
            },
            MockCell {
                count: Some(0),
                status: Unknown,
            },
            MockCell {
                count: Some(0),
                status: Unknown,
            },
            MockCell {
                count: Some(0),
                status: Unknown,
            },
            MockCell {
                count: Some(0),
                status: Unknown,
            },
            MockCell {
                count: Some(0),
                status: Unknown,
            },
            MockCell {
                count: Some(1),
                status: Unknown,
            },
            MockCell {
                count: Some(3),
                status: Unknown,
            },
            MockCell {
                count: None,
                status: Unknown,
            },
            MockCell {
                count: Some(3),
                status: Unknown,
            },
            MockCell {
                count: Some(0),
                status: Unknown,
            },
            MockCell {
                count: Some(0),
                status: Unknown,
            },
            MockCell {
                count: Some(0),
                status: Unknown,
            },
            MockCell {
                count: Some(0),
                status: Unknown,
            },
            MockCell {
                count: Some(0),
                status: Unknown,
            },
            MockCell {
                count: Some(1),
                status: Unknown,
            },
            MockCell {
                count: None,
                status: Unknown,
            },
            MockCell {
                count: Some(3),
                status: Unknown,
            },
            MockCell {
                count: None,
                status: Unknown,
            },
            MockCell {
                count: Some(0),
                status: Unknown,
            },
            MockCell {
                count: Some(0),
                status: Unknown,
            },
            MockCell {
                count: Some(0),
                status: Unknown,
            },
            MockCell {
                count: Some(0),
                status: Unknown,
            },
            MockCell {
                count: Some(0),
                status: Unknown,
            },
            MockCell {
                count: Some(1),
                status: Unknown,
            },
            MockCell {
                count: Some(1),
                status: Unknown,
            },
            MockCell {
                count: Some(2),
                status: Unknown,
            },
            MockCell {
                count: Some(1),
                status: Unknown,
            },
            MockCell {
                count: Some(0),
                status: Unknown,
            },
            MockCell {
                count: Some(1),
                status: Unknown,
            },
            MockCell {
                count: Some(1),
                status: Unknown,
            },
            MockCell {
                count: Some(1),
                status: Unknown,
            },
            MockCell {
                count: Some(0),
                status: Unknown,
            },
            MockCell {
                count: Some(0),
                status: Unknown,
            },
            MockCell {
                count: Some(0),
                status: Unknown,
            },
            MockCell {
                count: Some(0),
                status: Unknown,
            },
            MockCell {
                count: Some(0),
                status: Unknown,
            },
            MockCell {
                count: Some(0),
                status: Unknown,
            },
            MockCell {
                count: Some(1),
                status: Unknown,
            },
            MockCell {
                count: None,
                status: Unknown,
            },
            MockCell {
                count: Some(1),
                status: Unknown,
            },
            MockCell {
                count: Some(0),
                status: Unknown,
            },
            MockCell {
                count: Some(0),
                status: Unknown,
            },
            MockCell {
                count: Some(0),
                status: Unknown,
            },
            MockCell {
                count: Some(0),
                status: Unknown,
            },
            MockCell {
                count: Some(0),
                status: Unknown,
            },
        ],
        width: 9,
        length: 9,
        mines: 10,
    }}

    #[test]
    fn test_display() {
        assert_eq!(
            get_inst().to_string(),
            "\
Dimensions: 9 x 9
Flagged: 0 / 10
💣 ❔ ❔ 💣 ❔ ❔ ❔ ❔ ❔ 
💣 ❔ ❔ ❔ ❔ ❔ ❔ ❔ ❔ 
💣 ❔ ❔ ❔ ❔ ❔ 💣 ❔ ❔ 
❔ ❔ ❔ ❔ ❔ ❔ ❔ 💣 ❔ 
❔ ❔ ❔ ❔ ❔ ❔ ❔ 💣 ❔ 
❔ ❔ ❔ ❔ ❔ ❔ 💣 ❔ 💣 
❔ ❔ ❔ ❔ ❔ ❔ ❔ ❔ ❔ 
❔ ❔ ❔ ❔ ❔ ❔ ❔ ❔ ❔ 
❔ ❔ 💣 ❔ ❔ ❔ ❔ ❔ ❔ 
"
        );
    }

    #[test]
    fn test_solve() {
        let mut inst = get_inst();
        inst.solve().unwrap();
        assert_eq!(
            inst.to_string(),
            "\
Dimensions: 9 x 9
Flagged: 10 / 10
🚩 2. 1. 🚩 1. 0. 0. 0. 0. 
🚩 3. 1. 1. 1. 1. 1. 1. 0. 
🚩 2. 0. 0. 0. 1. 🚩 2. 1. 
1. 1. 0. 0. 0. 1. 3. 🚩 2. 
0. 0. 0. 0. 0. 1. 3. 🚩 3. 
0. 0. 0. 0. 0. 1. 🚩 3. 🚩 
0. 0. 0. 0. 0. 1. 1. 2. 1. 
0. 1. 1. 1. 0. 0. 0. 0. 0. 
0. 1. 🚩 1. 0. 0. 0. 0. 0. 
"
        )
    }
}