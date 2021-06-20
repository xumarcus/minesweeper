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

use std::cmp::max;

use ordered_float::NotNan;

fn new_board(board: &Vec<Status>, config: Config) -> Vec<Status> {
    let square_status = |idx, status| {
        config
            .square(idx)
            .filter(move |cidx| board[*cidx] == status)
    };

    let mut new_board: Vec<Status> = board.clone();
    for (idx, status) in board.iter().enumerate() {
        if let Status::Known(count) = status {
            let unknowns = square_status(idx, Status::Unknown).count();
            let flaggeds = square_status(idx, Status::Flagged).count();
            if *count == flaggeds {
                for cidx in square_status(idx, Status::Unknown) {
                    new_board[cidx] = Status::Marked;
                }
            } else if *count == unknowns + flaggeds {
                for cidx in square_status(idx, Status::Unknown) {
                    new_board[cidx] = Status::Flagged;
                }
            }
        }
    }
    new_board
}

pub trait Minesweeper: fmt::Display {
    // These getters/setters needed for abstraction
    fn get_config(&self) -> Config;
    fn get_board(&self) -> &Vec<Status>;
    fn set_board(&mut self, board: Vec<Status>) -> MsResult<()>;

    // Depends on implementation
    fn reveal(&mut self, idx: usize) -> MsResult<()>;

    // TODO use logger
    fn solve(&mut self) -> MsResult<()> {
        println!("{}", self);
        while let Some((idx, p)) = self.solve_next()? {
            println!("Guess {:?}: {:.3}", self.get_config().as_rc(idx), p);
            self.reveal(idx)?;
            println!("{}", self);
        }
        Ok(())
    }

    // 1.0f64 is exact
    fn solve_next(&mut self) -> MsResult<Option<(usize, f64)>> {
        let config = self.get_config();

        if let Some((idx, _)) = self
            .get_board()
            .iter()
            .enumerate()
            .find(|(_, status)| status == &&Status::Marked)
        {
            return Ok(Some((idx, 1.0)));
        }

        let board = self.get_board();
        if let Status::Known(_) = board[config.center()] {
            let mut next_board = new_board(&board, config);
            if board == &next_board {
                let count_status =
                    |status| board.iter().filter(|status_| **status_ == status).count();
                let not_flaggeds = config.mines - count_status(Status::Flagged);
                let all_unknowns = count_status(Status::Unknown);
                let base_prob = NotNan::new((not_flaggeds as f64) / (all_unknowns as f64));
                let mut prob = vec![None; config.size()];
                for (idx, status) in board.iter().enumerate() {
                    if let Status::Known(count) = status {
                        let square_unknowns = config
                            .square(idx)
                            .filter(|cidx| board[*cidx] == Status::Unknown)
                            .count();
                        let p = NotNan::new((*count as f64) / (square_unknowns as f64)).ok();
                        for idx_sq in config.square(idx) {
                            prob[idx_sq] = max(prob[idx_sq], p);
                        }
                    }
                }
                Ok(board
                    .iter()
                    .enumerate()
                    .filter(|(_, status)| status == &&Status::Unknown)
                    .map(|(idx, _)| {
                        let p = prob[idx].unwrap_or(base_prob.expect("`all_unknowns` != 0"));
                        (idx, p)
                    })
                    .min_by_key(|(_, p)| *p)
                    .map(|(idx, p)| (idx, p.into_inner())))
            } else {
                let mut next_next = new_board(&next_board, config);
                while next_board != next_next {
                    next_board = next_next;
                    next_next = new_board(&next_board, config);
                }
                self.set_board(next_board)?;
                self.solve_next()
            }
        } else {
            Ok(Some((config.center(), 1.0)))
        }
    }
}
