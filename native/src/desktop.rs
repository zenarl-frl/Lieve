use windows_sys::Win32::{Foundation::*,Graphics::Gdi::*,UI::WindowsAndMessaging::*};

fn covers(window:RECT,monitor:RECT,caption:bool)->bool {
    !caption && window.right>window.left && window.bottom>window.top
        && window.left<=monitor.left+2 && window.top<=monitor.top+2
        && window.right>=monitor.right-2 && window.bottom>=monitor.bottom-2
}
pub unsafe fn fullscreen_over(hwnd:HWND)->bool {
    let foreground=GetForegroundWindow();
    if foreground.is_null() || foreground==hwnd || foreground==GetShellWindow() || IsWindowVisible(foreground)==0 || IsIconic(foreground)!=0 {return false;}
    let mut class=[0u16;64];let count=GetClassNameW(foreground,class.as_mut_ptr(),class.len() as i32);
    if matches!(String::from_utf16_lossy(&class[..count.max(0) as usize]).as_str(),"Progman"|"WorkerW"|"Shell_TrayWnd"|"Shell_SecondaryTrayWnd") {return false;}
    let mut window=std::mem::zeroed::<RECT>();let mut monitor=std::mem::zeroed::<MONITORINFO>();monitor.cbSize=std::mem::size_of::<MONITORINFO>() as u32;
    if GetWindowRect(foreground,&mut window)==0 || GetMonitorInfoW(MonitorFromWindow(hwnd,MONITOR_DEFAULTTONEAREST),&mut monitor)==0 {return false;}
    covers(window,monitor.rcMonitor,(GetWindowLongPtrW(foreground,GWL_STYLE) as u32&WS_CAPTION)==WS_CAPTION)
}

#[cfg(test)] mod tests {
    use super::*;
    fn rect(l:i32,t:i32,r:i32,b:i32)->RECT {RECT{left:l,top:t,right:r,bottom:b}}
    #[test] fn fullscreen_requires_monitor_coverage_not_maximized_work_area() {
        let monitor=rect(0,0,1920,1080);
        assert!(covers(monitor,monitor,false));assert!(!covers(monitor,monitor,true));
        assert!(!covers(rect(0,0,1920,1040),monitor,false));assert!(!covers(rect(1920,0,3840,1080),monitor,false));
        assert!(covers(rect(1,1,1919,1079),monitor,false));
    }
    #[test] fn negative_origin_and_spanning_fullscreen_windows() {
        let monitor=rect(-1920,-200,0,880);
        assert!(covers(monitor,monitor,false));assert!(covers(rect(-1920,-200,1920,1080),monitor,false));
        assert!(!covers(rect(0,0,1920,1080),monitor,false));
    }
}
