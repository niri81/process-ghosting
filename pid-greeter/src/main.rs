use std::{thread::sleep, time::Duration};

use windows::{Win32::System::Threading::*, core::*};

fn main() -> Result<()> {
    unsafe {
        println!("Hello from PID {}", GetCurrentProcessId());
    }

    sleep(Duration::from_secs(15));

    Ok(())
}
