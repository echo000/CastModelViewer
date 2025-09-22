use std::path::Path;

/// Method to open a url in the user's default browser.
pub fn open_url<U: AsRef<str>>(url: U) {
    let url = url.as_ref();

    #[cfg(target_os = "windows")]
    {
        use std::ffi::OsStr;
        use std::os::windows::ffi::OsStrExt;

        use windows_sys::Win32::UI::Shell::*;
        use windows_sys::Win32::UI::WindowsAndMessaging::*;

        let url: Vec<u16> = OsStr::new(url).encode_wide().chain(Some(0x0)).collect();

        // SAFETY: The pointer to url lives as long as the call does, and is checked that it's a valid string,
        // in this case we do not care whether or not the call succeeds or fails.
        unsafe {
            ShellExecuteW(
                std::ptr::null_mut(),
                std::ptr::null(),
                url.as_ptr(),
                std::ptr::null(),
                std::ptr::null(),
                SW_SHOWNORMAL,
            )
        };
    }

    #[cfg(target_os = "macos")]
    {
        use std::process::Command;

        let result = Command::new("open").arg(url).output();

        debug_assert!(result.is_ok());
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        use std::process::Command;

        let result = Command::new("xdg-open").arg(url).output();

        debug_assert!(result.is_ok());
    }
}

/// Opens a folder in the users file explorer, creating the folder first if it doesn't exist.
pub fn open_folder<F: AsRef<Path>>(folder: F) {
    let folder = folder.as_ref();
    let dirs = std::fs::create_dir_all(folder);

    debug_assert!(dirs.is_ok());

    let folder = folder.to_string_lossy().into_owned();

    #[cfg(target_os = "windows")]
    {
        use std::os::windows::ffi::OsStrExt;

        use windows_sys::Win32::UI::Shell::*;
        use windows_sys::Win32::UI::WindowsAndMessaging::*;

        let folder: Vec<u16> = std::fs::canonicalize(folder)
            .unwrap_or_default()
            .as_os_str()
            .encode_wide()
            .chain(Some(0x0))
            .collect();

        // SAFETY: The pointer to url lives as long as the call does, and is checked that it's a valid string,
        // in this case we do not care whether or not the call succeeds or fails.
        unsafe {
            ShellExecuteW(
                std::ptr::null_mut(),
                std::ptr::null(),
                folder.as_ptr(),
                std::ptr::null(),
                std::ptr::null(),
                SW_SHOWNORMAL,
            )
        };
    }

    #[cfg(not(target_os = "windows"))]
    {
        use std::process::Command;

        let mut command = Command::new(if cfg!(target_os = "macos") {
            "open"
        } else {
            "xdg-open"
        });

        command.arg(folder);

        let result = command.output();

        debug_assert!(result.is_ok());
    }
}
