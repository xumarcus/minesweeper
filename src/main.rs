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
    let diff = match env::args().nth(1).as_deref() {
        Some("beginner") => Difficulty::Beginner,
        Some("intermediate") => Difficulty::Intermediate,
        Some("expert") => Difficulty::Expert,
        _ => Difficulty::Beginner
    };

    let seed = env::args().nth(2).and_then(|s| s.parse::<u64>().ok());
    let logger = match seed {
        None => SimpleLogger::new().with_level(log::LevelFilter::Debug),
        _ => SimpleLogger::new()
    };
    logger.init()?;
    
    let inst = MockMinesweeper::from_difficulty(diff, seed);
    let mut solver = Solver::new(inst);
    let res = solver.solve();
    log::info!("{}", solver);
    res.map_err(From::from)
}

/*
fn main() {
    for _ in 0..2000 {
        let inst = MockMinesweeper::from_difficulty(Difficulty::Expert, None);
        Solver::new(inst).solve();
    }
}
*/