use minesweeper::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut inst = MockMinesweeper::new(Config::from_difficulty(Difficulty::Beginner));
    inst.solve()?;
    Ok(())
}