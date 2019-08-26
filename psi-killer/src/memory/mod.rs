mod sysrq;

use std::io::Result;

fn low_mem_trigger() {}

fn out_of_memory_trigger() -> Result<()> {
    sysrq::trigger_oom_killer()
}
