extern crate log;
extern crate psi;
extern crate simplelog;

mod memory;

use std::time::Duration;

use log::*;
use simplelog::*;

use psi::error::*;
use psi::*;

fn main() -> Result<()> {
    SimpleLogger::init(LevelFilter::Trace, Config::default())?;
    let mut monitor = PsiMonitor::new()?;
    monitor.add_trigger(MemoryTrigger::some(
        Duration::from_millis(5),
        Duration::from_millis(500),
    ))?;
    loop {
        let psi_event = monitor.wait_single()?;
        debug!("{:?}", psi_event.stats);
    }
}
