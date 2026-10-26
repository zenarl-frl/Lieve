use std::{ptr::{null,null_mut},time::{Duration,Instant}};
use windows_sys::Win32::{Foundation::*,Graphics::Gdi::*,System::Threading::*,UI::WindowsAndMessaging::*};

pub fn interval(hz:u32)->Duration {Duration::from_secs_f64(1./if hz>=30 {hz.min(240) as f64}else{60.})}
pub fn next_deadline(previous:Instant,now:Instant,period:Duration)->Instant {
    let elapsed=now.saturating_duration_since(previous);
    let steps=(elapsed.as_nanos()/period.as_nanos()+1).min(u32::MAX as u128) as u32;
    previous+period*steps
}

pub struct FrameClock {timer:HANDLE,active:bool,period:Duration,next:Instant,fallback:bool}
impl FrameClock {
    pub unsafe fn new()->Self {
        let timer=CreateWaitableTimerExW(null(),null(),CREATE_WAITABLE_TIMER_HIGH_RESOLUTION,TIMER_ALL_ACCESS);
        Self {timer,active:false,period:interval(60),next:Instant::now(),fallback:timer.is_null()}
    }
    pub fn handle(&self)->HANDLE {if self.active && !self.fallback {self.timer}else{null_mut()}}
    pub unsafe fn monitor_changed(&mut self,hwnd:HWND) {
        let mut info=std::mem::zeroed::<MONITORINFOEXW>();
        info.monitorInfo.cbSize=std::mem::size_of::<MONITORINFOEXW>() as u32;
        let mut mode=std::mem::zeroed::<DEVMODEW>(); mode.dmSize=std::mem::size_of::<DEVMODEW>() as u16;
        let hz=if GetMonitorInfoW(MonitorFromWindow(hwnd,MONITOR_DEFAULTTOPRIMARY),(&mut info as *mut MONITORINFOEXW).cast())!=0
            && EnumDisplaySettingsW(info.szDevice.as_ptr(),ENUM_CURRENT_SETTINGS,&mut mode)!=0 {mode.dmDisplayFrequency}else{60};
        self.period=interval(hz);
    }
    pub unsafe fn start(&mut self,hwnd:HWND) {
        if self.active {return;} // Retargeting must never postpone an armed frame.
        self.active=true; self.next=Instant::now()+self.period;
        if self.fallback || !self.arm() {self.fallback=true; SetTimer(hwnd,2,16,None);}
    }
    unsafe fn arm(&self)->bool {
        let delay=self.next.saturating_duration_since(Instant::now());
        let due=-((delay.as_nanos()/100).max(1).min(i64::MAX as u128) as i64);
        SetWaitableTimerEx(self.timer,&due,0,None,null(),null(),0)!=0
    }
    pub unsafe fn complete(&mut self,hwnd:HWND) {
        if !self.active || self.fallback {return;}
        self.next=next_deadline(self.next,Instant::now(),self.period);
        if !self.arm() {self.fallback=true; SetTimer(hwnd,2,16,None);}
    }
    pub unsafe fn stop(&mut self,hwnd:HWND) {
        self.active=false; KillTimer(hwnd,2);
        if !self.timer.is_null() {CancelWaitableTimer(self.timer);}
    }
}
impl Drop for FrameClock {
    fn drop(&mut self) {unsafe {if !self.timer.is_null() {CancelWaitableTimer(self.timer);CloseHandle(self.timer);}}}
}

#[cfg(test)] mod tests {
    use super::*;
    #[test] fn cadence_tracks_display_rate_and_rejects_unknown_rates() {
        assert_eq!(interval(0),interval(60)); assert_eq!(interval(1),interval(60));
        assert!((interval(120).as_secs_f64()-1./120.).abs()<1e-8);
    }
    #[test] fn late_frames_skip_missed_deadlines_without_drift_or_catch_up_bursts() {
        let start=Instant::now(); let period=Duration::from_millis(10);
        assert_eq!(next_deadline(start,start+Duration::from_millis(3),period),start+period);
        assert_eq!(next_deadline(start,start+Duration::from_millis(35),period),start+period*4);
        assert_eq!(next_deadline(start,start+period,period),start+period*2);
    }
}
