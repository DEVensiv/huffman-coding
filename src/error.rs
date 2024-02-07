use std::{fmt::Display, io};

#[derive(Debug)]
pub enum Error {
    // tried to operate on more bits then were available
    NoBits,
    IoError(io::Error),
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::NoBits => None,
            Error::IoError(err) => Some(err),
        }
    }

    fn description(&self) -> &str {
        "description() is deprecated; use Display"
    }

    fn cause(&self) -> Option<&dyn std::error::Error> {
        self.source()
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::NoBits => writeln!(f, "tried to operate on more bits than were available"),
            Error::IoError(err) => err.fmt(f),
        }
    }
}

impl From<io::Error> for Error {
    fn from(value: io::Error) -> Self {
        Error::IoError(value)
    }
}

impl From<Error> for io::Error {
    fn from(val: Error) -> Self {
        match val {
            Error::NoBits => io::Error::other(val),
            Error::IoError(err) => err,
        }
    }
}
