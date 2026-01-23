use windows::Win32::Foundation::{CloseHandle, HANDLE};

pub struct SafeHandle(pub HANDLE);

impl Drop for SafeHandle {
    fn drop(&mut self) {
        if !self.0.is_invalid() {
            unsafe {
                let _ = CloseHandle(self.0);
                // Optional: log that the handle was closed if verbose is high
            }
        }
    }
}

impl From<HANDLE> for SafeHandle {
    fn from(value: HANDLE) -> Self {
        Self(value)
    }
}
