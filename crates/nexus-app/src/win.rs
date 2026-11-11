//! Small Windows integrations: dark title bar matching the midnight backdrop.

#[cfg(windows)]
pub fn style_title_bar(frame: &eframe::Frame) {
    use raw_window_handle::{HasWindowHandle, RawWindowHandle};
    use windows_sys::Win32::Graphics::Dwm::DwmSetWindowAttribute;
    let Ok(handle) = frame.window_handle() else { return };
    let RawWindowHandle::Win32(h) = handle.as_raw() else { return };
    let hwnd = h.hwnd.get() as windows_sys::Win32::Foundation::HWND;
    unsafe {
        // DWMWA_USE_IMMERSIVE_DARK_MODE = 20
        let dark: i32 = 1;
        DwmSetWindowAttribute(hwnd, 20, &dark as *const i32 as *const _, 4);
        // DWMWA_CAPTION_COLOR = 35 (COLORREF 0x00BBGGRR) — #070C18
        let caption: u32 = 0x00180C07;
        DwmSetWindowAttribute(hwnd, 35, &caption as *const u32 as *const _, 4);
        // DWMWA_BORDER_COLOR = 34
        let border: u32 = 0x00342317;
        DwmSetWindowAttribute(hwnd, 34, &border as *const u32 as *const _, 4);
    }
}

#[cfg(not(windows))]
pub fn style_title_bar(_frame: &eframe::Frame) {}
