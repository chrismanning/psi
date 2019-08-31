use std::collections::hash_map::*;
use std::fmt;
use std::fs::{File, OpenOptions};
use std::io::SeekFrom::Start;
use std::io::{Read, Seek, Write};
use std::os::unix::io::*;

use epoll::*;
use log::*;

use crate::error::*;
use crate::psi::*;
use crate::trigger::*;

/// PSI event
pub struct PsiEvent {
    /// PSI stats as read after the event fired
    pub stats: Psi,
    /// The trigger this event fired for
    pub trigger: Trigger,
    /// Opaque ID of the trigger this event fired for
    pub id: TriggerId,
}

impl fmt::Display for PsiEvent {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "event triggered, stats: {}, trigger: {}",
            self.stats, self.trigger
        )
    }
}

struct PsiTriggerTarget {
    trigger: Trigger,
    file: File,
    buf: String,
}

/// PSI monitor
///
/// Watches for changes to resource pressure based on user-defined thresholds.
pub struct PsiMonitor {
    epoll_fd: RawFd,
    triggers: HashMap<RawFd, PsiTriggerTarget>,
}

impl PsiMonitor {
    /// Create a PsiMonitor instance.
    pub fn new() -> Result<Self> {
        let epoll_fd = create(false)?;
        Ok(PsiMonitor {
            epoll_fd,
            triggers: HashMap::new(),
        })
    }

    /// Add a trigger to the monitor
    ///
    /// Registers a threshold with the kernel and uses epoll to handle events
    /// the kernel produces when the threshold is reached.
    pub fn add_trigger(&mut self, trigger: Trigger) -> Result<TriggerId> {
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(&trigger.target_file_path)?;
        info!("registering {}", &trigger);
        debug!("trigger: {:?}", trigger.generate_trigger());
        debug!(
            "trigger bytes: {:?}",
            trigger.generate_trigger().as_bytes_with_nul()
        );
        file.write_all(trigger.generate_trigger().as_bytes_with_nul())?;
        info!("successfully registered {}", trigger);
        let raw_fd = file.as_raw_fd();

        // add event to epoll
        let event = Event::new(Events::EPOLLPRI, raw_fd as u64);
        ctl(self.epoll_fd, ControlOptions::EPOLL_CTL_ADD, raw_fd, event)?;

        let target = PsiTriggerTarget {
            file,
            trigger,
            buf: String::with_capacity(128),
        };
        self.triggers.insert(raw_fd, target);
        Ok(TriggerId { raw_fd })
    }

    /// Wait for a PSI pressure event to fire based on some previously added trigger(s).
    pub fn wait_single(&mut self) -> Result<PsiEvent> {
        debug!("waiting for psi event");
        let mut event_buf = [Event { events: 0, data: 0 }];
        let n = wait(self.epoll_fd, -1, &mut event_buf)?;
        assert_eq!(n, 1);
        let event = event_buf[0];
        let fd = event.data as RawFd;
        assert_ne!(fd, 0);
        match self.triggers.get_mut(&fd) {
            None => Err(UnregisteredEvent),
            Some(target) => {
                info!("psi event triggered: {}", target.trigger);
                if event.events & Events::EPOLLERR.bits() != 0 {
                    error!("error on watched psi file");
                    return Err(PsiTriggerFileError);
                }
                target.buf.clear();
                debug!("reading contents of file {:?}", target.file.file_path());
                target.file.seek(Start(0))?;
                target.file.read_to_string(&mut target.buf)?;
                debug!("psi: {}", target.buf);

                let stats = target.buf.parse()?;
                Ok(PsiEvent {
                    stats,
                    trigger: target.trigger.clone(),
                    id: TriggerId { raw_fd: fd },
                })
            }
        }
    }
}

/// ID for a specific trigger
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct TriggerId {
    raw_fd: RawFd,
}
