pub mod error;
pub mod monitor;
pub mod trigger;

use error::*;
pub use error::{PsiError, Result};
pub use monitor::PsiMonitor;
pub use trigger::MemoryTrigger;

use std::fmt;
use std::str::FromStr;

// https://www.kernel.org/doc/html/latest/accounting/psi.html

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum PsiKind {
    Memory,
    IO,
    CPU,
}

impl fmt::Display for PsiKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            PsiKind::Memory => write!(f, "memory"),
            PsiKind::IO => write!(f, "io"),
            PsiKind::CPU => write!(f, "cpu"),
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum PsiLine {
    Some,
    Full,
}

impl fmt::Display for PsiLine {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            PsiLine::Some => write!(f, "some"),
            PsiLine::Full => write!(f, "full"),
        }
    }
}

impl FromStr for PsiLine {
    type Err = PsiError;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "some" => Ok(PsiLine::Some),
            "full" => Ok(PsiLine::Full),
            _ => Err(ParseError::UnexpectedTerm(s.to_string()).into()),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct PsiStats {
    pub line: PsiLine,
    pub avg10: f32,
    pub avg60: f32,
    pub avg300: f32,
    pub total: u64,
}

impl PsiStats {
    fn parse_stat<E: Into<PsiError>, T: FromStr<Err = E>>(key: &str, term: &str) -> Result<T> {
        let v: Vec<&str> = term.splitn(2, |c| c == '=').collect();
        if v.len() == 2 && v[0] == key {
            v[1].parse().map_err(|e: E| e.into())
        } else {
            Err(PsiError::PsiParseError(ParseError::UnexpectedTerm(
                term.to_string(),
            )))
        }
    }
}

impl fmt::Display for PsiStats {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{} avg10={} avg60={} avg300={} total={}",
            self.line, self.avg10, self.avg60, self.avg300, self.total
        )
    }
}

impl FromStr for PsiStats {
    type Err = PsiError;

    fn from_str(s: &str) -> Result<Self> {
        let terms: Vec<&str> = s.split_ascii_whitespace().collect();
        let line = terms[0].parse()?;
        let avg10 = PsiStats::parse_stat(
            "avg10",
            terms
                .get(1)
                .ok_or(ParseError::UnexpectedTerm(s.to_string()))?,
        )?;
        let avg60 = PsiStats::parse_stat(
            "avg60",
            terms
                .get(2)
                .ok_or(ParseError::UnexpectedTerm(s.to_string()))?,
        )?;
        let avg300 = PsiStats::parse_stat(
            "avg300",
            terms
                .get(3)
                .ok_or(ParseError::UnexpectedTerm(s.to_string()))?,
        )?;
        let total = PsiStats::parse_stat(
            "total",
            terms
                .get(4)
                .ok_or(ParseError::UnexpectedTerm(s.to_string()))?,
        )?;
        Ok(PsiStats {
            line,
            avg10,
            avg60,
            avg300,
            total,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_parse_full() {
        let line = "full avg10=0.16 avg60=0.00 avg300=0.00 total=27787674";
        let stats = PsiStats::from_str(line);
        assert_eq!(
            stats.unwrap(),
            PsiStats {
                line: PsiLine::Full,
                avg10: 0.16f32,
                avg60: 0f32,
                avg300: 0f32,
                total: 27787674,
            }
        );
    }

    #[test]
    fn should_parse_some() {
        let line = "some avg10=0.16 avg60=0.00 avg300=0.00 total=27787674";
        let stats = PsiStats::from_str(line);
        assert_eq!(
            stats.unwrap(),
            PsiStats {
                line: PsiLine::Some,
                avg10: 0.16f32,
                avg60: 0f32,
                avg300: 0f32,
                total: 27787674,
            }
        );
    }
}
