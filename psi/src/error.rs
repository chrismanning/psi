use std::error::Error;
use std::fmt;
use std::io::ErrorKind;
use std::num::{ParseFloatError, ParseIntError};

pub(crate) use std::result::Result as StdResult;

pub type Result<T> = StdResult<T, PsiError>;

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
    LoggingInitError(log::SetLoggerError),
}

#[derive(Debug)]
pub enum ParseError {
    TotalParseError(ParseIntError),
    AvgParseError(ParseFloatError),
    UnexpectedTerm(String),
}

impl From<ParseError> for PsiError {
    fn from(e: ParseError) -> Self {
        PsiError::PsiParseError(e)
    }
}

impl Error for PsiError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        use ParseError::*;
        use PsiError::*;
        match self {
            IoError(e) => Some(e),
            PsiParseError(e) => match e {
                TotalParseError(e) => Some(e),
                AvgParseError(e) => Some(e),
                UnexpectedTerm(_) => None,
            },
            InvalidThreshold(e) => Some(e),
            LoggingInitError(e) => Some(e),
            _ => None,
        }
    }
}

impl fmt::Display for PsiError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use ParseError::*;
        use PsiError::*;
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
                    TotalParseError(_) => write!(f, "error parsing psi total"),
                    AvgParseError(_) => write!(f, "error parsing psi avg"),
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
            ErrorKind::InvalidInput => PsiError::InvalidThreshold(e),
            _ => PsiError::IoError(e),
        }
    }
}

impl From<ParseFloatError> for PsiError {
    fn from(e: ParseFloatError) -> Self {
        PsiError::PsiParseError(ParseError::AvgParseError(e))
    }
}

impl From<ParseIntError> for PsiError {
    fn from(e: ParseIntError) -> Self {
        PsiError::PsiParseError(ParseError::TotalParseError(e))
    }
}

impl From<log::SetLoggerError> for PsiError {
    fn from(e: log::SetLoggerError) -> Self {
        PsiError::LoggingInitError(e)
    }
}
