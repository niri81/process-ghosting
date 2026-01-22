use windows::{Win32::System::Threading::*, core::*};

fn main() -> Result<()> {
    unsafe {
        println!("Hello from PID {}", GetCurrentProcessId());
    }

    Ok(())
}
