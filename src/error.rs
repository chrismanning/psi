use std::error::Error;
use std::fmt;
use std::io::ErrorKind;
use std::num::{ParseFloatError, ParseIntError};

pub(crate) use std::result::Result as StdResult;
pub(crate) use ParseError::*;
pub(crate) use PsiError::*;

pub type Result<T> = StdResult<T, PsiError>;

/// Error type for PSI
#[derive(Debug)]
pub enum PsiError {
    IoError(std::io::Error),
    PsiParseError(ParseError),
    InvalidThreshold(std::io::Error),
    UnexpectedTriggerEvent {
        expected_kind: crate::PsiKind,
        expected_line: crate::PsiLine,
    },
    UnregisteredEvent,
    PsiTriggerFileError,
    LoggingInitError(log::SetLoggerError),
}

/// Error type for PSI parsing
#[derive(Debug)]
pub enum ParseError {
    TotalParseError(ParseIntError),
    AvgParseError(ParseFloatError),
    UnexpectedTerm(String),
    MissingLine(crate::PsiLine),
}

impl From<ParseError> for PsiError {
    fn from(e: ParseError) -> Self {
        PsiParseError(e)
    }
}

impl Error for PsiError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            IoError(e) => Some(e),
            PsiParseError(e) => match e {
                TotalParseError(_) => None,
                AvgParseError(_) => None,
                UnexpectedTerm(_) => None,
                MissingLine(_) => None,
            },
            InvalidThreshold(e) => Some(e),
            LoggingInitError(e) => Some(e),
            _ => None,
        }
    }
}

impl fmt::Display for PsiError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.source() {
            None => match self {
                UnexpectedTriggerEvent {
                    expected_kind,
                    expected_line,
                } => write!(
                    f,
                    "unexpected trigger event; expected {} {}",
                    expected_kind, expected_line
                ),
                UnregisteredEvent => write!(f, "unregistered event triggered"),
                PsiParseError(p) => match p {
                    UnexpectedTerm(t) => write!(f, "unexpected psi term '{}'", t),
                    TotalParseError(e) => write!(f, "error parsing psi total: {}", e),
                    AvgParseError(e) => write!(f, "error parsing psi avg: {}", e),
                    MissingLine(line) => write!(f, "missing line '{}'", line),
                },
                _ => write!(f, "unknown error"),
            },
            Some(e) => write!(f, "{}", e),
        }
    }
}

impl From<std::io::Error> for PsiError {
    fn from(e: std::io::Error) -> Self {
        match e.kind() {
            ErrorKind::InvalidInput => InvalidThreshold(e),
            _ => IoError(e),
        }
    }
}

impl From<ParseFloatError> for PsiError {
    fn from(e: ParseFloatError) -> Self {
        PsiParseError(AvgParseError(e))
    }
}

impl From<ParseIntError> for PsiError {
    fn from(e: ParseIntError) -> Self {
        PsiParseError(TotalParseError(e))
    }
}

impl From<log::SetLoggerError> for PsiError {
    fn from(e: log::SetLoggerError) -> Self {
        LoggingInitError(e)
    }
}
