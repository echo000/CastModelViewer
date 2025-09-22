use std::os::windows::ffi::OsStrExt;
use std::sync::OnceLock;

use windows_sys::Win32::System::LibraryLoader::*;
use windows_sys::Win32::UI::Shell::*;
use windows_sys::Win32::UI::WindowsAndMessaging::*;

/// The executable icon.
static WINDOWS_ICON: OnceLock<usize> = OnceLock::new();

/// Gets the embedded executable icon on windows.
pub fn windows_icon() -> HICON {
    *WINDOWS_ICON.get_or_init(|| {
        let Ok(path) = std::env::current_exe() else {
            return 0;
        };

        let path: Vec<u16> = path.as_os_str().encode_wide().chain(Some(0x0)).collect();

        // SAFETY:
        // Path is checked to be a valid u16cstring above. The result is checked for errors where
        // 1 = not an icon or exe.
        // 0 = any other error.
        // While this obviously leaks the icon, the icon won't change for the lifetime of the program and can be used in more
        // than one window during the lifetime of the program.
        let icon = unsafe { ExtractIconW(GetModuleHandleW(std::ptr::null()), path.as_ptr(), 0) };

        if icon as usize <= 1 {
            return 0;
        }

        icon as usize
    }) as _
}
