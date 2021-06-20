use thiserror::Error;
#[derive(Clone, Debug, Error)]
pub enum MinesweeperError {
    #[error("InvalidParameters")]
    InvalidParameters,
    #[error("IsAlreadySolved")]
    IsAlreadySolved,
    #[error("IsAlreadyFlagged")]
    IsAlreadyFlagged,
    #[error("RevealedBomb")]
    RevealedBomb,
    #[error("SetInvalidStatus")]
    SetInvalidStatus,
    #[error("SetSameStatus")]
    SetSameStatus,
}

pub type MsResult<T> = Result<T, MinesweeperError>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Status {
    Flagged,
    Known(usize),
    Marked,
    Unknown,
}

#[derive(Clone, Debug)]
pub enum Difficulty {
    Beginner,
    Intermediate,
    Expert,
}