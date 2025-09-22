use std::path::Path;

/// A custom icon that can be displayed in the application interface.
#[derive(Debug, Clone)]
pub struct Icon {
    width: u32,
    height: u32,
    buffer: Vec<u8>,
}

impl Icon {
    /// Constructs a new icon, from the given width, height, and buffer of `R8G8B8A8_UNORM` data.
    pub fn new(width: u32, height: u32, buffer: Vec<u8>) -> Self {
        Self {
            width,
            height,
            buffer,
        }
    }

    /// Creates a rounded version of this icon with the given border radius.
    pub fn rounded(mut self, radius: f32) -> Self {
        let width = self.width as f32;
        let height = self.height as f32;

        let radius = if radius < 0.0 {
            0.0
        } else {
            radius.min(width / 2.0).min(height / 2.0)
        };

        if radius == 0.0 {
            return self;
        }

        for (index, pixel) in self.buffer.chunks_exact_mut(4).enumerate() {
            let y = index / self.width as usize;
            let x = index % self.width as usize;

            let px = x as f32 + 0.5;
            let py = y as f32 + 0.5;

            let pos_x = px - width / 2.0;
            let pos_y = py - height / 2.0;

            let qx = pos_x.abs() - (width / 2.0 - radius);
            let qy = pos_y.abs() - (height / 2.0 - radius);

            let q_max_x = qx.max(0.0);
            let q_max_y = qy.max(0.0);
            let len = (q_max_x * q_max_x + q_max_y * q_max_y).sqrt();

            let min_max_q = qx.max(qy).min(0.0);
            let sdf = min_max_q + len - radius;

            let fraction = (0.5 - sdf).clamp(0.0, 1.0);
            let alpha = pixel[3] as f32;

            pixel[3] = (alpha * fraction) as u8;
        }

        self
    }

    /// Extracts an icon from whatever file the given path points to.
    #[cfg(target_os = "windows")]
    pub fn extract<P: AsRef<Path>>(path: P) -> Option<Self> {
        use std::os::windows::ffi::OsStrExt;

        use windows_sys::Win32::Foundation::*;
        use windows_sys::Win32::Graphics::Gdi::*;
        use windows_sys::Win32::UI::Shell::*;
        use windows_sys::Win32::UI::WindowsAndMessaging::*;

        use porter_utils::VecExt;

        let path = path.as_ref();
        let path: Vec<u16> = path.as_os_str().encode_wide().chain(Some(0x0)).collect();

        let extract_icon = |path: Vec<u16>| unsafe {
            let mut icon: HICON = std::ptr::null_mut();
            let sizes = [64, 48, 32, 16];

            for size in sizes {
                if SHDefExtractIconW(path.as_ptr(), 0, 0, &mut icon, std::ptr::null_mut(), size)
                    == S_OK
                {
                    return icon;
                }
            }

            icon
        };

        let icon = extract_icon(path);

        if icon.is_null() {
            return None;
        }

        let mut icon_info = ICONINFO::default();

        unsafe { GetIconInfo(icon, &mut icon_info) };

        let mut bitmap = BITMAP::default();

        let bitmap_size = size_of_val(&bitmap);
        let bitmap_pointer = &mut bitmap as *mut _;

        unsafe { GetObjectW(icon_info.hbmColor, bitmap_size as _, bitmap_pointer as _) };

        let bitmap_header = BITMAPINFOHEADER {
            biSize: size_of::<BITMAPINFOHEADER>() as _,
            biPlanes: 1,
            biBitCount: bitmap.bmBitsPixel,
            biWidth: bitmap.bmWidth,
            // Ensure image is oriented correctly, bitmaps are upside down.
            biHeight: -bitmap.bmHeight,
            biCompression: BI_RGB,
            ..Default::default()
        };

        let mut bitmap_info = BITMAPINFO {
            bmiHeader: bitmap_header,
            ..Default::default()
        };

        let Ok(mut buffer) =
            Vec::try_new_with_value(0u8, 4 * bitmap.bmWidth as usize * bitmap.bmHeight as usize)
        else {
            unsafe { DeleteObject(icon_info.hbmColor) };
            unsafe { DeleteObject(icon_info.hbmMask) };
            unsafe { DestroyIcon(icon) };

            return None;
        };

        let hdc = unsafe { GetDC(std::ptr::null_mut()) };

        unsafe {
            GetDIBits(
                hdc,
                icon_info.hbmColor,
                0,
                bitmap.bmHeight as _,
                buffer.as_mut_ptr() as _,
                &mut bitmap_info,
                DIB_RGB_COLORS,
            )
        };

        unsafe { ReleaseDC(std::ptr::null_mut(), hdc) };

        unsafe { DeleteObject(icon_info.hbmColor) };
        unsafe { DeleteObject(icon_info.hbmMask) };
        unsafe { DestroyIcon(icon) };

        // Conversion from B8G8R8A8 -> R8G8B8A8.
        for pixel in buffer.chunks_exact_mut(4) {
            pixel.swap(0, 2);
            pixel[3] = 255;
        }

        Some(Self {
            width: bitmap.bmWidth as _,
            height: bitmap.bmHeight as _,
            buffer,
        })
    }

    /// Extracts an icon from whatever file the given path points to.
    #[cfg(not(target_os = "windows"))]
    pub fn extract<P: AsRef<Path>>(path: P) -> Option<Self> {
        let _ = path;
        None
    }
}

impl From<Icon> for iced::widget::image::Handle {
    fn from(value: Icon) -> Self {
        Self::from_rgba(value.width, value.height, value.buffer)
    }
}
