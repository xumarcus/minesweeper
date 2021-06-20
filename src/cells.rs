use super::*;

pub trait Cell {
    fn get(&self) -> Option<usize>;
    fn get_status(&self) -> Status;
    fn set_ok(&mut self) -> MsResult<()>;
    fn set_flag(&mut self) -> MsResult<()>;
    fn set_count(&mut self, count: usize) -> MsResult<()>;
}

#[derive(Clone, Debug)]
pub struct MockCell(Status)

impl fmt::Display for MockCell {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_bomb {
            match self.status {
                Status::Flagged => write!(f, "🚩")?,
                Status::Marked => unreachable!("Wrong solution"),
                Status::Unknown => write!(f, "💣")?,
                Status::Known(_) => unreachable!("Is bomb"),
            }
        } else {
            match self.status {
                Status::Flagged => unreachable!("Wrong flag"),
                Status::Marked => write!(f, "✔️")?,
                Status::Unknown => write!(f, "❔")?,
                Status::Known(x) => write!(f, "{}.", x)?,
            }
        }
        Ok(())
    }
}

impl Cell for MockCell {
    fn get(&self) -> Option<usize> {
        match self.status {
            Status::Known(x) => Some(x),
            _ => None
        }
    }

    fn get_status(&self) -> Status {
        self.status.clone()
    }

    fn set_ok(&mut self) -> MsResult<()> {
        match self.status {
            Status::Unknown => {
                self.status = Status::Marked;
                Ok(())
            },
            Status::Marked => Err(MinesweeperError::SetSameStatus),
            _ => Err(MinesweeperError::SetInvalidStatus)
        }
    }

    fn set_flag(&mut self) -> MsResult<()> {
        match self.status {
            Status::Unknown => {
                self.status = Status::Flagged;
                Ok(())
            },
            Status::Flagged => Err(MinesweeperError::SetSameStatus),
            _ => Err(MinesweeperError::SetInvalidStatus)
        }
    }

    fn set_count(&mut self, count: usize) - > MsResult<()> {
        if self.is_bomb() {
            Err(MinesweeperError::SetInvalidStatus)
        } else {
            match self.status {
                Status::Known(_) => Err(MinesweeperError::SetSameStatus),
                _ => {
                    self.status = Status::Known(count);
                    Ok(())
                }
            }
        }
    }
}

impl MockCell {
    pub fn new(is_bomb: bool) -> Self {
        Self { is_bomb, status: Status::Unknown }
    }

    pub fn is_bomb(&self) -> bool {
        self.is_bomb
    }
}