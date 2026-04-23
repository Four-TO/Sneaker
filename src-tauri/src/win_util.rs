#[cfg(windows)]
pub fn apply_click_through(hwnd: isize, enable: bool) {
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        GetWindowLongPtrW, SetWindowLongPtrW, GWL_EXSTYLE, WS_EX_LAYERED, WS_EX_TRANSPARENT,
    };
    unsafe {
        let h = hwnd as windows_sys::Win32::Foundation::HWND;
        let mut ex = GetWindowLongPtrW(h, GWL_EXSTYLE);
        if enable {
            ex |= (WS_EX_LAYERED | WS_EX_TRANSPARENT) as isize;
        } else {
            ex &= !(WS_EX_TRANSPARENT as isize);
        }
        SetWindowLongPtrW(h, GWL_EXSTYLE, ex);
    }
}

#[cfg(not(windows))]
pub fn apply_click_through(_hwnd: isize, _enable: bool) {}
