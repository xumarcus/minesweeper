use minesweeper::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let inst = MockMinesweeper::new(Config::from_difficulty(Difficulty::Beginner));
    println!("{}", inst);
    Ok(())
}