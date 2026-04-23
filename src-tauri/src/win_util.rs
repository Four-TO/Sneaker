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

#[cfg(windows)]
pub fn apply_tool_window(hwnd: isize, enable: bool) {
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        GetWindowLongPtrW, SetWindowLongPtrW, ShowWindow, GWL_EXSTYLE,
        SW_HIDE, SW_SHOWNA, WS_EX_APPWINDOW, WS_EX_TOOLWINDOW,
    };
    unsafe {
        let h = hwnd as windows_sys::Win32::Foundation::HWND;
        let was_visible = windows_sys::Win32::UI::WindowsAndMessaging::IsWindowVisible(h) != 0;
        let mut ex = GetWindowLongPtrW(h, GWL_EXSTYLE);
        if was_visible { ShowWindow(h, SW_HIDE); }
        if enable {
            ex |= WS_EX_TOOLWINDOW as isize;
            ex &= !(WS_EX_APPWINDOW as isize);
        } else {
            ex &= !(WS_EX_TOOLWINDOW as isize);
            ex |= WS_EX_APPWINDOW as isize;
        }
        SetWindowLongPtrW(h, GWL_EXSTYLE, ex);
        if was_visible { ShowWindow(h, SW_SHOWNA); }
    }
}

#[cfg(not(windows))]
pub fn apply_tool_window(_hwnd: isize, _enable: bool) {}
