extern crate log;
extern crate psi;
extern crate simplelog;

mod memory;

use std::time::Duration;

use log::*;
use simplelog::*;

use psi::error::*;
use psi::*;

use memory::*;

fn main() -> Result<()> {
    SimpleLogger::init(LevelFilter::Trace, Config::default())?;
    let mut monitor = PsiMonitor::new()?;
    let low_mem_id = monitor.add_trigger(
        Trigger::new_builder()
            .memory()
            .full()
            .stall(Duration::from_millis(50))
            .window(Duration::from_millis(500))
            .build(),
    )?;
    let oom_id = monitor.add_trigger(
        Trigger::new_builder()
            .memory()
            .full()
            .stall(Duration::from_millis(100))
            .window(Duration::from_millis(500))
            .build(),
    )?;
    loop {
        let psi_event = monitor.wait_single()?;
        debug!("{:?}", psi_event.stats);
        if oom_id == psi_event.id {
            info!("out of memory threshold reached");
            trigger_oom_killer()?;
        } else if low_mem_id == psi_event.id {
            info!("low memory threshold reached");
            low_mem_handler()?;
        }
    }
}
