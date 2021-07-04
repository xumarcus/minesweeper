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

use minesweeper::*;

use std::env;

use simple_logger::SimpleLogger;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let diff = env::args()
        .nth(1)
        .as_deref()
        .and_then(|s| s.parse::<Difficulty>().ok())
        .unwrap_or_default();
    let seed = env::args().nth(2).and_then(|s| s.parse::<u64>().ok());
    let logger = match seed {
        None => SimpleLogger::new().with_level(log::LevelFilter::Debug),
        _ => SimpleLogger::new(),
    };
    logger.init()?;

    let config = Config::from_difficulty(diff, seed);
    let solver = Solver::new(config);
    let mut inst = MockMinesweeper::new(config);
    while let Some(_) = solver.solve_next(&mut inst)? {
        println!("{}", ShowMinesweeper(&inst));
    }
    Ok(())
}
