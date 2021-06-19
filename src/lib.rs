use rand::{
    self,
    distributions::{Distribution, Uniform},
};
use std::cmp::{max, min};
use std::fmt;
use thiserror::Error;

#[derive(Clone, Debug, Error)]
pub enum MinesweeperError {
    #[error("InvalidParameters")]
    InvalidParameters,
    #[error("IsAlreadySolved")]
    IsAlreadySolved,
    #[error("AttemptRevealBomb")]
    AttemptRevealBomb,
}

#[derive(Clone, Debug)]
pub enum Difficulty {
    Beginner,
    Intermediate,
    Expert,
    Custom(usize, usize, usize),
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Status {
    Flagged,
    ToSolve,
    Unknown,
    Visible,
}

#[derive(Clone, Debug)]
pub struct Cell {
    count: Option<usize>,
    status: Status,
}

impl Default for Cell {
    fn default() -> Self {
        Self {
            count: Some(0),
            status: Status::Unknown,
        }
    }
}

impl fmt::Display for Cell {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.count {
            Some(x) => match self.status {
                Status::Flagged => write!(f, "?!")?, // Wrong flag
                Status::ToSolve => write!(f, "{}!", x)?,
                Status::Unknown => write!(f, "❔")?,
                Status::Visible => write!(f, "{}.", x)?,
            },
            None => match self.status {
                Status::Flagged => write!(f, "🚩")?,
                Status::ToSolve => write!(f, "??")?, // Wrong sol
                Status::Unknown => write!(f, "💣")?,
                Status::Visible => unreachable!(), // Bombs not visible
            },
        }
        Ok(())
    }
}

impl Cell {
    fn get(&self) -> Option<usize> {
        (self.status == Status::Visible).then(|| self.count.unwrap())
    }
}

#[derive(Clone, Debug)]
pub struct Minesweeper {
    board: Vec<Cell>,
    width: usize,
    length: usize,
}

impl fmt::Display for Minesweeper {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Dimensions: {} x {}", self.width, self.length)?;
        writeln!(
            f,
            "Flagged: {} / {}",
            self.count_status(Status::Flagged),
            self.mines()
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

impl Minesweeper {
    fn square(&self, idx: usize) -> impl Iterator<Item = usize> {
        let len = self.length;
        let row = idx / len;
        let col = idx % len;
        (max(1, row) - 1..=min(self.width - 1, row + 1))
            .flat_map(move |r| (max(1, col) - 1..=min(len - 1, col + 1)).map(move |c| r * len + c))
    }

    fn count_status(&self, status: Status) -> usize {
        self.board
            .iter()
            .filter(|cell| cell.status == status)
            .count()
    }

    fn count_status_square(&self, idx: usize, status: Status) -> usize {
        self.square(idx)
            .filter(|cidx| self.board[*cidx].status == status)
            .count()
    }

    fn mines(&self) -> usize {
        self.board
            .iter()
            .filter(|cell| cell.count.is_none())
            .count()
    }

    pub fn new(width: usize, length: usize, mines: usize) -> Result<Self, MinesweeperError> {
        if width * length <= mines {
            return Err(MinesweeperError::InvalidParameters);
        }
        let mut inst = Self {
            board: vec![Cell::default(); width * length],
            width,
            length,
        };
        let mut rng = rand::thread_rng();
        let w_gen = Uniform::from(0..width);
        let l_gen = Uniform::from(0..length);
        for _ in 0..mines {
            loop {
                let idx = w_gen.sample(&mut rng) * length + l_gen.sample(&mut rng);
                if inst.board[idx].count.is_some() {
                    inst.board[idx].count = None;
                    for cidx in inst.square(idx) {
                        if let Some(count) = inst.board[cidx].count.as_mut() {
                            *count += 1;
                        }
                    }
                    break;
                }
            }
        }
        Ok(inst)
    }

    #[rustfmt::skip]
    pub fn from_difficulty(diff: Difficulty) -> Result<Self, MinesweeperError> {
        match diff {
            Difficulty::Beginner     => Self::new( 9,  9, 10),
            Difficulty::Intermediate => Self::new(16, 16, 40),
            Difficulty::Expert       => Self::new(30, 16, 99),
            Difficulty::Custom(width, length, mines) => Self::new(width, length, mines)
        }
    }

    pub fn reveal(&mut self, idx: usize) -> Result<(), MinesweeperError> {
        self.board[idx]
            .count
            .map(|x| {
                self.board[idx].status = Status::Visible;
                if x == 0 {
                    let mut st = vec![idx];
                    while let Some(cur) = st.pop() {
                        self.board[cur].status = Status::Visible;
                        if let Some(0) = self.board[cur].count {
                            st.extend(
                                self.square(cur)
                                    .filter(|cidx| self.board[*cidx].status != Status::Visible),
                            );
                        }
                    }
                }
            })
            .ok_or(MinesweeperError::AttemptRevealBomb)
    }

    // 1.0f64 is exact
    pub fn solve(&mut self) -> Result<(usize, f64), MinesweeperError> {
        loop {
            let n = self.board.len();
            let mut to_solve: Vec<Option<bool>> = vec![None; n];
            for (idx, cell) in self.board.iter().enumerate() {
                if let Some(count) = cell.get() {
                    let unknowns = self.count_status_square(idx, Status::Unknown);
                    let flags = self.count_status_square(idx, Status::Flagged);
                    if unknowns == count - flags {
                        for cidx in self.square(idx) {
                            if self.board[cidx].status == Status::Unknown {
                                to_solve[cidx] = Some(true);
                            }
                        }
                    } else if unknowns == count {
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
        self.board
            .iter()
            .enumerate()
            .find(|(_, cell)| cell.status == Status::ToSolve)
            .map(|(idx, _)| (idx, 1.0))
            .or_else(|| {
                let p = (self.mines() as f64) / (self.count_status(Status::Unknown) as f64);
                self.board
                    .iter()
                    .enumerate()
                    .find(|(_, cell)| cell.status == Status::Unknown)
                    .map(|(idx, _)| (idx, p))
            })
            .ok_or(MinesweeperError::IsAlreadySolved)
    }
}
