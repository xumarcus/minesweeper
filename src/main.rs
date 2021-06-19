use minesweeper::{Difficulty, Minesweeper};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut inst = Minesweeper::from_difficulty(Difficulty::Beginner);
    println!("Initial");
    println!("{}", inst);
    println!("{:?}", inst);
    while let Ok((idx, p)) = inst.solve_next() {
        println!("Guess {:?}: {:.3}", inst.as_rc(idx), p);
        inst.reveal(idx)?;
        println!("{}", inst);
        println!("{:?}", inst);
    }
    Ok(())
}
