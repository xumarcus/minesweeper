use thiserror::Error;

#[derive(Clone, Debug, Error)]
pub enum MinesweeperError {
    #[error("InvalidParameters")]
    InvalidParameters,
    #[error("FlaggedButNotBomb")]
    FlaggedButNotBomb,
    #[error("RevealedBomb")]
    RevealedBomb,
}

pub type MsResult<T> = Result<T, MinesweeperError>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Status {
    Flagged,
    Known(usize),
    Marked,
    Unknown,
}

impl Default for Status {
    fn default() -> Self {
        Self::Unknown
    }
}

#[derive(Clone, Debug)]
pub enum Difficulty {
    Beginner,
    Intermediate,
    Expert,
}