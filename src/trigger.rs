use std::ffi::CString;
use std::fmt;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::*;

const CPU_PRESSURE_FILEPATH: &'static str = "/proc/pressure/cpu";
const IO_PRESSURE_FILEPATH: &'static str = "/proc/pressure/io";
const MEMORY_PRESSURE_FILEPATH: &'static str = "/proc/pressure/memory";

/// PSI trigger
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Trigger {
    pub kind: PsiKind,
    pub line: PsiLine,
    pub target_file_path: PathBuf,
    pub threshold: TriggerThreshold,
}

impl Trigger {
    /// Start building a trigger
    pub fn new_builder() -> TriggerBuilder {
        TriggerBuilder
    }

    pub(crate) fn generate_trigger(&self) -> CString {
        let mut buf = Vec::<u8>::with_capacity(32);
        match self.line {
            PsiLine::Some => write!(&mut buf, "some ").unwrap(),
            PsiLine::Full => write!(&mut buf, "full ").unwrap(),
        }
        let t = &self.threshold;
        write!(&mut buf, "{} {}", t.stall.as_micros(), t.window.as_micros()).unwrap();
        CString::new(buf).unwrap()
    }
}

impl fmt::Display for Trigger {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "psi {} trigger on '{}' line with threshold: {}",
            self.kind, self.line, self.threshold
        )
    }
}

pub struct TriggerBuilder;

impl TriggerBuilder {
    pub fn kind(self, kind: PsiKind) -> TriggerBuilderKind {
        match kind {
            PsiKind::CPU => self.cpu(),
            PsiKind::IO => self.io(),
            PsiKind::Memory => self.memory(),
        }
    }

    pub fn cpu(self) -> TriggerBuilderKind {
        TriggerBuilderKind {
            kind: PsiKind::CPU,
            target_file_path: Path::new(CPU_PRESSURE_FILEPATH).to_path_buf(),
        }
    }

    pub fn io(self) -> TriggerBuilderKind {
        TriggerBuilderKind {
            kind: PsiKind::IO,
            target_file_path: Path::new(IO_PRESSURE_FILEPATH).to_path_buf(),
        }
    }

    pub fn memory(self) -> TriggerBuilderKind {
        TriggerBuilderKind {
            kind: PsiKind::Memory,
            target_file_path: Path::new(MEMORY_PRESSURE_FILEPATH).to_path_buf(),
        }
    }
}

pub struct TriggerBuilderKind {
    kind: PsiKind,
    target_file_path: PathBuf,
}

impl TriggerBuilderKind {
    pub fn line(self, line: PsiLine) -> TriggerBuilderLine {
        TriggerBuilderLine {
            line,
            kind: self.kind,
            target_file_path: self.target_file_path,
        }
    }

    pub fn some(self) -> TriggerBuilderLine {
        self.line(PsiLine::Some)
    }

    pub fn full(self) -> TriggerBuilderLine {
        self.line(PsiLine::Full)
    }
}

pub struct TriggerBuilderLine {
    kind: PsiKind,
    target_file_path: PathBuf,
    line: PsiLine,
}

impl TriggerBuilderLine {
    pub fn stall(self, stall: Duration) -> TriggerBuilderStall {
        TriggerBuilderStall {
            stall,
            kind: self.kind,
            target_file_path: self.target_file_path,
            line: self.line,
        }
    }

    pub fn threshold(self, threshold: TriggerThreshold) -> TriggerBuilderStaging {
        TriggerBuilderStaging {
            threshold,
            kind: self.kind,
            target_file_path: self.target_file_path,
            line: self.line,
        }
    }
}

pub struct TriggerBuilderStall {
    kind: PsiKind,
    target_file_path: PathBuf,
    line: PsiLine,
    stall: Duration,
}

impl TriggerBuilderStall {
    pub fn window(self, window: Duration) -> TriggerBuilderStaging {
        TriggerBuilderStaging {
            kind: self.kind,
            target_file_path: self.target_file_path,
            line: self.line,
            threshold: TriggerThreshold {
                window,
                stall: self.stall,
            },
        }
    }
}

pub struct TriggerBuilderStaging {
    kind: PsiKind,
    target_file_path: PathBuf,
    line: PsiLine,
    threshold: TriggerThreshold,
}

impl TriggerBuilderStaging {
    pub fn build(self) -> Trigger {
        Trigger {
            kind: self.kind,
            target_file_path: self.target_file_path,
            line: self.line,
            threshold: self.threshold,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct TriggerThreshold {
    pub stall: Duration,
    pub window: Duration,
}

impl fmt::Display for TriggerThreshold {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}us stall in {}us window",
            self.stall.as_micros(),
            self.window.as_micros()
        )
    }
}
