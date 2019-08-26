use std::ffi::CString;
use std::fmt;
use std::io::Write;
use std::path::PathBuf;
use std::time::Duration;

use crate::error::*;
use crate::*;

const MEMORY_PRESSURE_FILEPATH: &'static str = "/proc/pressure/memory";

pub struct Trigger {
    pub kind: PsiKind,
    pub line: PsiLine,
    pub trigger_filepath: PathBuf,
    pub threshold: TriggerThreshold,
}

impl Trigger {
    pub fn generate_trigger(&self) -> CString {
        let mut buf = Vec::<u8>::with_capacity(32);
        match self.line {
            PsiLine::Some => write!(&mut buf, "some ").unwrap(),
            PsiLine::Full => write!(&mut buf, "full ").unwrap(),
        }
        let t = &self.threshold;
        write!(&mut buf, "{} {}", t.stall.as_micros(), t.window.as_micros()).unwrap();
        CString::new(buf).unwrap()
    }

    pub fn get_psi_stats(&self, psi: &str) -> Result<PsiStats> {
        let pat = match self.line {
            PsiLine::Some => "some ",
            PsiLine::Full => "full ",
        };
        psi.lines()
            .find(|line| line.starts_with(pat))
            .ok_or(PsiError::UnexpectedTriggerEvent {
                expected_kind: self.kind,
                expected_line: self.line,
            })
            .and_then(|line| line.parse())
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

pub struct MemoryTrigger;

impl MemoryTrigger {
    pub fn some(stall: Duration, window: Duration) -> Trigger {
        Self::new(PsiLine::Some, stall, window)
    }

    pub fn full(stall: Duration, window: Duration) -> Trigger {
        Self::new(PsiLine::Full, stall, window)
    }

    pub fn new(line: PsiLine, stall: Duration, window: Duration) -> Trigger {
        Trigger {
            line,
            kind: PsiKind::Memory,
            trigger_filepath: MEMORY_PRESSURE_FILEPATH.into(),
            threshold: TriggerThreshold { stall, window },
        }
    }
}
