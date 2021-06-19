use minesweeper::{Difficulty, Minesweeper};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut inst = Minesweeper::from_difficulty(Difficulty::Beginner)?;
    println!("{}", inst);
    while let Ok((idx, p)) = inst.solve() {
        println!("[{}]: {:.3}", idx, p);
        inst.reveal(idx)?;
        println!("{}", inst);
    }
    Ok(())
}
