use std::fmt;
use std::fs::{read_link, File, OpenOptions};
use std::io::Read;
use std::os::unix::io::*;
use std::path::PathBuf;
use std::str::FromStr;
use std::time::Duration;

use crate::error::*;

pub(crate) const CPU_PRESSURE_FILEPATH: &'static str = "/proc/pressure/cpu";
pub(crate) const IO_PRESSURE_FILEPATH: &'static str = "/proc/pressure/io";
pub(crate) const MEMORY_PRESSURE_FILEPATH: &'static str = "/proc/pressure/memory";

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum PsiKind {
    Memory,
    IO,
    CPU,
}

impl PsiKind {
    pub fn read_psi(&self) -> Result<AllPsiStats> {
        let path = match self {
            PsiKind::Memory => MEMORY_PRESSURE_FILEPATH,
            PsiKind::IO => IO_PRESSURE_FILEPATH,
            PsiKind::CPU => CPU_PRESSURE_FILEPATH,
        };
        let mut file = OpenOptions::new().read(true).open(path)?;
        let mut buf = String::with_capacity(256);
        file.read_to_string(&mut buf)?;
        let all = buf
            .lines()
            .map(|line| line.parse())
            .collect::<Result<Vec<Psi>>>()?;
        let some = all
            .iter()
            .find(|psi| psi.line == PsiLine::Some)
            .ok_or(MissingLine(PsiLine::Some))?
            .clone();
        let full = all
            .iter()
            .find(|psi| psi.line == PsiLine::Full)
            .ok_or(MissingLine(PsiLine::Full))?
            .clone();
        Ok(AllPsiStats { some, full })
    }

    pub fn read_psi_line(&self, line: PsiLine) -> Result<Psi> {
        let all = self.read_psi()?;
        match line {
            PsiLine::Some => Ok(all.some),
            PsiLine::Full => Ok(all.full),
        }
    }
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
            _ => Err(UnexpectedTerm(s.to_string()).into()),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Psi {
    pub line: PsiLine,
    pub avg10: f32,
    pub avg60: f32,
    pub avg300: f32,
    pub total: Duration,
}

impl Psi {
    fn parse_stat<E: Into<PsiError>, T: FromStr<Err = E>>(key: &str, term: &str) -> Result<T> {
        let v: Vec<&str> = term.splitn(2, |c| c == '=').collect();
        if v.len() == 2 && v[0] == key {
            v[1].parse::<T>().map_err(E::into)
        } else {
            Err(PsiParseError(UnexpectedTerm(term.to_string())))
        }
    }
}

impl fmt::Display for Psi {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{} avg10={} avg60={} avg300={} total={}",
            self.line,
            self.avg10,
            self.avg60,
            self.avg300,
            self.total.as_micros()
        )
    }
}

impl FromStr for Psi {
    type Err = PsiError;

    fn from_str(s: &str) -> Result<Self> {
        let terms: Vec<&str> = s.split_ascii_whitespace().collect();
        let line = terms[0].parse()?;
        let avg10 = Psi::parse_stat("avg10", terms.get(1).ok_or(UnexpectedTerm(s.to_string()))?)?;
        let avg60 = Psi::parse_stat("avg60", terms.get(2).ok_or(UnexpectedTerm(s.to_string()))?)?;
        let avg300 = Psi::parse_stat("avg300", terms.get(3).ok_or(UnexpectedTerm(s.to_string()))?)?;
        let total_stall =
            Psi::parse_stat("total", terms.get(4).ok_or(UnexpectedTerm(s.to_string()))?)?;
        Ok(Psi {
            line,
            avg10,
            avg60,
            avg300,
            total: Duration::from_micros(total_stall),
        })
    }
}

#[derive(Debug, PartialEq)]
pub struct AllPsiStats {
    pub some: Psi,
    pub full: Psi,
}

impl fmt::Display for AllPsiStats {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "{}", self.some)?;
        write!(f, "{}", self.full)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_parse_full() {
        let line = "full avg10=0.16 avg60=0.00 avg300=0.00 total=27787674";
        let stats = Psi::from_str(line);
        assert_eq!(
            stats.unwrap(),
            Psi {
                line: PsiLine::Full,
                avg10: 0.16f32,
                avg60: 0f32,
                avg300: 0f32,
                total: Duration::from_micros(27787674),
            }
        );
    }

    #[test]
    fn should_parse_some() {
        let line = "some avg10=0.16 avg60=0.00 avg300=0.00 total=27787674";
        let stats = Psi::from_str(line);
        assert_eq!(
            stats.unwrap(),
            Psi {
                line: PsiLine::Some,
                avg10: 0.16f32,
                avg60: 0f32,
                avg300: 0f32,
                total: Duration::from_micros(27787674),
            }
        );
    }
}

pub(crate) trait FilePath {
    fn file_path(&self) -> StdResult<PathBuf, std::io::Error>;
}

impl FilePath for File {
    fn file_path(&self) -> StdResult<PathBuf, std::io::Error> {
        let raw_fd = self.as_raw_fd();
        let fd_link_path = format!("/proc/self/fd/{}", raw_fd);
        read_link(fd_link_path)
    }
}
