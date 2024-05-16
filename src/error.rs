use std::io::Error as IoError;

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[allow(dead_code)]
#[derive(Debug)]
pub enum Error {
    Other(&'static str),
    NoChange,
    NotFound,
    UnexpectedEof,
    Io(IoError),
}

impl From<IoError> for Error {
    fn from(err: IoError) -> Self {
        Self::Io(err)
    }
}

impl Error {
    fn as_str(&self) -> &'static str {
        use Error::*;
        match self {
            NoChange => "no changes to be made",
            NotFound => "entity not found",
            UnexpectedEof => "unexpected end of file",
            _ => "unknown",
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Error::*;
        match self {
            NoChange => write!(f, "{}", NoChange.as_str()),
            NotFound => write!(f, "{}", NotFound.as_str()),
            UnexpectedEof => write!(f, "{}", UnexpectedEof),
            Io(e) => write!(f, "{}", e),
            Other(msg) => write!(f, "{}", msg),
        }
    }
}
