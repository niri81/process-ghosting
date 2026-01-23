use std::{thread::sleep, time::Duration};

use windows::Win32::System::Threading::GetCurrentProcessId;

fn main() {
    unsafe {
        println!("Hello from PID {}", GetCurrentProcessId());
    }

    sleep(Duration::from_secs(15));
}
