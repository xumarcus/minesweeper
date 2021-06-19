#![feature(test)]
extern crate test;

use ordered_float::NotNan;
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
                Status::Flagged => unreachable!(), // Wrong flag
                Status::ToSolve => write!(f, "{}!", x)?,
                Status::Unknown => write!(f, "❔")?,
                Status::Visible => write!(f, "{}.", x)?,
            },
            None => match self.status {
                Status::Flagged => write!(f, "🚩")?,
                Status::ToSolve => unreachable!(), // Wrong sol
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
        let (row, col) = self.as_rc(idx);
        let len = self.length; // Copy length from self
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

    #[inline]
    pub fn as_rc(&self, idx: usize) -> (usize, usize) {
        (idx / self.length, idx % self.length)
    }

    #[inline]
    pub fn from_rc(&self, row: usize, col: usize) -> usize {
        row * self.length + col
    }

    pub fn mines(&self) -> usize {
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
                let row = w_gen.sample(&mut rng);
                let col = l_gen.sample(&mut rng);
                if row == width / 2 && col == length / 2 {
                    continue; // keep center revealable
                }
                let idx = inst.from_rc(row, col);
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
    pub fn from_difficulty(diff: Difficulty) -> Self {
        let result = match diff {
            Difficulty::Beginner     => Self::new( 9,  9, 10),
            Difficulty::Intermediate => Self::new(16, 16, 40),
            Difficulty::Expert       => Self::new(30, 16, 99),
        };
        result.unwrap()
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

#[cfg(test)]
mod tests {
    use super::{Status::*, *};
    use test::Bencher;

    fn get_inst() -> Minesweeper {
        Minesweeper {
        board: vec![
            Cell {
                count: None,
                status: Unknown,
            },
            Cell {
                count: Some(2),
                status: Unknown,
            },
            Cell {
                count: Some(1),
                status: Unknown,
            },
            Cell {
                count: None,
                status: Unknown,
            },
            Cell {
                count: Some(1),
                status: Unknown,
            },
            Cell {
                count: Some(0),
                status: Unknown,
            },
            Cell {
                count: Some(0),
                status: Unknown,
            },
            Cell {
                count: Some(0),
                status: Unknown,
            },
            Cell {
                count: Some(0),
                status: Unknown,
            },
            Cell {
                count: None,
                status: Unknown,
            },
            Cell {
                count: Some(3),
                status: Unknown,
            },
            Cell {
                count: Some(1),
                status: Unknown,
            },
            Cell {
                count: Some(1),
                status: Unknown,
            },
            Cell {
                count: Some(1),
                status: Unknown,
            },
            Cell {
                count: Some(1),
                status: Unknown,
            },
            Cell {
                count: Some(1),
                status: Unknown,
            },
            Cell {
                count: Some(1),
                status: Unknown,
            },
            Cell {
                count: Some(0),
                status: Unknown,
            },
            Cell {
                count: None,
                status: Unknown,
            },
            Cell {
                count: Some(2),
                status: Unknown,
            },
            Cell {
                count: Some(0),
                status: Unknown,
            },
            Cell {
                count: Some(0),
                status: Unknown,
            },
            Cell {
                count: Some(0),
                status: Unknown,
            },
            Cell {
                count: Some(1),
                status: Unknown,
            },
            Cell {
                count: None,
                status: Unknown,
            },
            Cell {
                count: Some(2),
                status: Unknown,
            },
            Cell {
                count: Some(1),
                status: Unknown,
            },
            Cell {
                count: Some(1),
                status: Unknown,
            },
            Cell {
                count: Some(1),
                status: Unknown,
            },
            Cell {
                count: Some(0),
                status: Unknown,
            },
            Cell {
                count: Some(0),
                status: Unknown,
            },
            Cell {
                count: Some(0),
                status: Unknown,
            },
            Cell {
                count: Some(1),
                status: Unknown,
            },
            Cell {
                count: Some(3),
                status: Unknown,
            },
            Cell {
                count: None,
                status: Unknown,
            },
            Cell {
                count: Some(2),
                status: Unknown,
            },
            Cell {
                count: Some(0),
                status: Unknown,
            },
            Cell {
                count: Some(0),
                status: Unknown,
            },
            Cell {
                count: Some(0),
                status: Unknown,
            },
            Cell {
                count: Some(0),
                status: Unknown,
            },
            Cell {
                count: Some(0),
                status: Unknown,
            },
            Cell {
                count: Some(1),
                status: Unknown,
            },
            Cell {
                count: Some(3),
                status: Unknown,
            },
            Cell {
                count: None,
                status: Unknown,
            },
            Cell {
                count: Some(3),
                status: Unknown,
            },
            Cell {
                count: Some(0),
                status: Unknown,
            },
            Cell {
                count: Some(0),
                status: Unknown,
            },
            Cell {
                count: Some(0),
                status: Unknown,
            },
            Cell {
                count: Some(0),
                status: Unknown,
            },
            Cell {
                count: Some(0),
                status: Unknown,
            },
            Cell {
                count: Some(1),
                status: Unknown,
            },
            Cell {
                count: None,
                status: Unknown,
            },
            Cell {
                count: Some(3),
                status: Unknown,
            },
            Cell {
                count: None,
                status: Unknown,
            },
            Cell {
                count: Some(0),
                status: Unknown,
            },
            Cell {
                count: Some(0),
                status: Unknown,
            },
            Cell {
                count: Some(0),
                status: Unknown,
            },
            Cell {
                count: Some(0),
                status: Unknown,
            },
            Cell {
                count: Some(0),
                status: Unknown,
            },
            Cell {
                count: Some(1),
                status: Unknown,
            },
            Cell {
                count: Some(1),
                status: Unknown,
            },
            Cell {
                count: Some(2),
                status: Unknown,
            },
            Cell {
                count: Some(1),
                status: Unknown,
            },
            Cell {
                count: Some(0),
                status: Unknown,
            },
            Cell {
                count: Some(1),
                status: Unknown,
            },
            Cell {
                count: Some(1),
                status: Unknown,
            },
            Cell {
                count: Some(1),
                status: Unknown,
            },
            Cell {
                count: Some(0),
                status: Unknown,
            },
            Cell {
                count: Some(0),
                status: Unknown,
            },
            Cell {
                count: Some(0),
                status: Unknown,
            },
            Cell {
                count: Some(0),
                status: Unknown,
            },
            Cell {
                count: Some(0),
                status: Unknown,
            },
            Cell {
                count: Some(0),
                status: Unknown,
            },
            Cell {
                count: Some(1),
                status: Unknown,
            },
            Cell {
                count: None,
                status: Unknown,
            },
            Cell {
                count: Some(1),
                status: Unknown,
            },
            Cell {
                count: Some(0),
                status: Unknown,
            },
            Cell {
                count: Some(0),
                status: Unknown,
            },
            Cell {
                count: Some(0),
                status: Unknown,
            },
            Cell {
                count: Some(0),
                status: Unknown,
            },
            Cell {
                count: Some(0),
                status: Unknown,
            },
        ],
        width: 9,
        length: 9,
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

    #[bench]
    #[should_panic]
    fn bench_random_beginner(b: &mut Bencher) {
        let mut solved = 0;
        let mut n = 0;
        b.iter(|| {
            let mut inst = Minesweeper::from_difficulty(Difficulty::Beginner);
            if inst.solve().is_ok() {
                solved += 1;
            }
            n += 1;
        });
        // cargo +nightly bench -- --nocapture
        println!("{:.3} ({} / {})", (solved as f64) / (n as f64), solved, n);
    }
}