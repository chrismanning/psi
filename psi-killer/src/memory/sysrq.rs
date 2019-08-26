use std::fs::OpenOptions;
use std::io::{Result, Write};

const SYSRQ_TRIGGER_PATH: &'static str = "/proc/sysrq-trigger";

pub fn trigger_oom_killer() -> Result<()> {
    let mut sysrq = OpenOptions::new().write(true).open(SYSRQ_TRIGGER_PATH)?;
    sysrq.write_all(&['f' as u8])
}
