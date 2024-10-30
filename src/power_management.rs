// power_management.rs

use windows::Win32::System::Power::{SetThreadExecutionState, ES_CONTINUOUS, ES_DISPLAY_REQUIRED, ES_SYSTEM_REQUIRED, ES_AWAYMODE_REQUIRED};

// Функція для підтримки активного стану системи
// #[allow(dead_code)]


pub fn keep_system_awake() {
    unsafe {
        SetThreadExecutionState(ES_CONTINUOUS | ES_DISPLAY_REQUIRED | ES_SYSTEM_REQUIRED | ES_AWAYMODE_REQUIRED);
    }
}
