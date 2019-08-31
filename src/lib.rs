//! Linux Pressure Stall Information (PSI) support for Rust.
//!
//! # About
//!
//! The Linux [Pressure Stall Information (PSI)] feature provides real-time
//! pressure information for CPU, IO and memory.
//! [psi] is a rust library for reading PSI and monitoring for pressure
//! thresholds on Linux 4.20+.
//!
//! # Example
//!
//! ```
//! use crate::psi::*;
//!
//! fn example() -> Result<()> {
//!     let all: AllPsiStats = PsiKind::Memory.read_psi()?;
//!     if all.some.avg10 > 0.1f32 {
//!         // do something
//!     }
//!     let full: Psi = PsiKind::Memory.read_psi_line(PsiLine::Full)?;
//!     if full.avg60 > 0.1f32 {
//!         // do something else
//!     }
//!     Ok(())
//! }
//! ```
//!
//! # Monitor Example
//!
//! ```
//! use std::time::Duration;
//! use crate::psi::*;
//!
//! fn example() -> Result<()> {
//!     let mut monitor = PsiMonitor::new()?;
//!     let oom_id = monitor.add_trigger(
//!         Trigger::new_builder()
//!             .memory()
//!             .full()
//!             .stall(Duration::from_millis(100))
//!             .window(Duration::from_millis(500))
//!             .build(),
//!     )?;
//!
//!     loop {
//!         let psi_event: PsiEvent = monitor.wait_single()?;
//!         // react to psi_event
//!     }
//! }
//! ```
//!
//! # TODO
//! * cgroup2 support
//!
//! [psi]: https://crates.io/crates/psi
//! [Pressure Stall Information (PSI)]: https://www.kernel.org/doc/html/latest/accounting/psi.html

pub mod error;
pub mod monitor;
pub mod psi;
pub mod trigger;

pub use crate::psi::{AllPsiStats, Psi, PsiKind, PsiLine};
pub use error::{PsiError, Result};
pub use monitor::{PsiEvent, PsiMonitor, TriggerId};
pub use trigger::Trigger;
