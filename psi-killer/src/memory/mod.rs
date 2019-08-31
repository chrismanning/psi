use std::fs::OpenOptions;
use std::io::{Result, Write};

use log::*;

const SYSRQ_TRIGGER_PATH: &'static str = "/proc/sysrq-trigger";

pub fn low_mem_handler() -> Result<()> {
    Ok(())
}

pub fn trigger_oom_killer() -> Result<()> {
    warn!("triggering oom killer");
    let mut sysrq = OpenOptions::new().write(true).open(SYSRQ_TRIGGER_PATH)?;
    sysrq.write_all(&['f' as u8])
}
