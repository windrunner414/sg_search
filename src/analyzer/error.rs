use std::fmt::{Debug, Display, Formatter};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    IO(std::io::Error),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match *self {
            Error::IO(ref e) => Display::fmt(&e, f),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match *self {
            Error::IO(ref e) => Some(e),
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::IO(e)
    }
}
