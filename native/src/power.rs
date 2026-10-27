use windows_sys::Win32::System::Power::{GetSystemPowerStatus, SYSTEM_POWER_STATUS};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Source { Ac, Battery, Unknown }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Snapshot {
    pub source: Source,
    pub present: Option<bool>,
    pub percent: Option<u8>,
    pub charging:bool,
}

impl Snapshot {
    fn from_raw(ac: u8, flags: u8, percent: u8) -> Self {
        let present = if flags == 255 { None } else { Some(flags & 128 == 0) };
        Self {
            source: match ac { 0 => Source::Battery, 1 => Source::Ac, _ => Source::Unknown },
            present,
            percent: if present == Some(false) { None } else { Some(percent).filter(|p| *p <= 100) },
            charging:present==Some(true) && ac==1 && flags!=255 && flags&8!=0,
        }
    }

    pub fn label(&self) -> String {
        if self.present == Some(false) { return "No battery".into(); }
        let level = self.percent.map(|p| format!("{p}%")).unwrap_or_else(|| "—%".into());
        match self.source {
            Source::Ac => format!("{level} · AC"),
            Source::Battery => level,
            Source::Unknown => "Power —".into(),
        }
    }
}

pub fn read() -> Snapshot {
    let mut status = unsafe { std::mem::zeroed::<SYSTEM_POWER_STATUS>() };
    if unsafe { GetSystemPowerStatus(&mut status) } == 0 {
        return Snapshot::from_raw(255, 255, 255);
    }
    Snapshot::from_raw(status.ACLineStatus, status.BatteryFlag, status.BatteryLifePercent)
}

pub struct Monitor {
    pub snapshot: Snapshot,
    warned: bool,
    last_source:Option<Source>,
    change:Option<Event>,
}
#[derive(Clone,Copy,Debug,PartialEq,Eq)] pub enum Event {Connected,Disconnected}

impl Monitor {
    pub fn new(snapshot: Snapshot) -> Self { Self { snapshot, warned: false,last_source:known_source(snapshot),change:None } }
    pub fn take_change(&mut self)->Option<Event> {self.change.take()}

    // Rearm only on AC or after recovery to 25%, not on noisy 20/21% readings.
    // Unknown readings retain the latch and never generate an alert.
    pub fn update(&mut self, snapshot: Snapshot) -> Option<u8> {
        if let Some(source)=known_source(snapshot) {
            if self.last_source.is_some_and(|old|old!=source) {self.change=Some(if source==Source::Ac {Event::Connected}else{Event::Disconnected});}
            self.last_source=Some(source);
        }else if snapshot.present==Some(false) {self.last_source=None;}
        self.snapshot = snapshot;
        if snapshot.source == Source::Ac || (snapshot.present == Some(true)
            && snapshot.source == Source::Battery && snapshot.percent.is_some_and(|p| p >= 25)) {
            self.warned = false;
        }
        if snapshot.present == Some(true) && snapshot.source == Source::Battery && !self.warned {
            if let Some(percent) = snapshot.percent.filter(|p| *p <= 20) {
                self.warned = true;
                return Some(percent);
            }
        }
        None
    }
}
fn known_source(snapshot:Snapshot)->Option<Source> {
    (snapshot.present==Some(true) && snapshot.source!=Source::Unknown).then_some(snapshot.source)
}
pub struct Subscription([windows_sys::Win32::System::Power::HPOWERNOTIFY;2]);
impl Subscription {
    pub unsafe fn new(hwnd:windows_sys::Win32::Foundation::HWND)->Self {
        use windows_sys::{core::GUID,Win32::System::Power::RegisterPowerSettingNotification};
        Self([RegisterPowerSettingNotification(hwnd,&GUID::from_u128(0x5d3e9a59_e9d5_4b00_a6bd_ff34ff516548),0),
            RegisterPowerSettingNotification(hwnd,&GUID::from_u128(0xa7ad8041_b45a_4cae_87a3_eecbb468a9e1),0)])
    }
}
impl Drop for Subscription {fn drop(&mut self) {for handle in self.0 {if handle!=0 {unsafe {windows_sys::Win32::System::Power::UnregisterPowerSettingNotification(handle);}}}}}

#[cfg(test)]
mod tests {
    use super::*;
    fn battery(percent: u8) -> Snapshot { Snapshot::from_raw(0, 0, percent) }
    #[test] #[ignore="registers real Windows power notifications on a hidden test window"]
    fn live_notification_registration() {
        use windows_sys::Win32::UI::WindowsAndMessaging::*;
        unsafe {
            let hwnd=CreateWindowExW(WS_EX_TOOLWINDOW,crate::wide("STATIC").as_ptr(),crate::wide("Lieve power subscription test").as_ptr(),WS_POPUP,0,0,1,1,std::ptr::null_mut(),std::ptr::null_mut(),std::ptr::null_mut(),std::ptr::null());
            assert!(!hwnd.is_null());let subscription=Subscription::new(hwnd);let registered=subscription.0.iter().all(|h|*h!=0);
            drop(subscription);DestroyWindow(hwnd);assert!(registered,"Both power-source and battery-percentage subscriptions must register");
            println!("Power source and battery percentage notification registration: PASS");
        }
    }

    #[test] fn handles_missing_and_unknown_batteries() {
        let absent = Snapshot::from_raw(1, 128, 255);
        assert_eq!(absent.present, Some(false));
        assert_eq!(absent.percent, None);
        assert_eq!(absent.label(), "No battery");
        let unknown = Snapshot::from_raw(255, 255, 255);
        assert_eq!(unknown.present, None);
        assert_eq!(unknown.percent, None);
        assert_eq!(unknown.label(), "Power —");
        assert_eq!(Snapshot::from_raw(1, 8, 76).label(), "76% · AC");
        assert_eq!(battery(101).percent, None);
    }
    #[test] fn source_changes_skip_startup_duplicates_and_bridge_unknown_readings() {
        let mut m=Monitor::new(battery(80));m.update(battery(80));assert_eq!(m.take_change(),None);
        m.update(Snapshot::from_raw(255,255,255));m.update(Snapshot::from_raw(1,8,80));assert_eq!(m.take_change(),Some(Event::Connected));
        assert!(m.snapshot.charging);m.update(Snapshot::from_raw(1,0,80));assert_eq!(m.take_change(),None);assert!(!m.snapshot.charging);
        m.update(battery(79));assert_eq!(m.take_change(),Some(Event::Disconnected));assert_eq!(m.take_change(),None);
        let mut unknown=Monitor::new(Snapshot::from_raw(255,255,255));unknown.update(battery(80));assert_eq!(unknown.take_change(),None);
    }

    #[test] fn warns_once_and_ignores_threshold_jitter() {
        let mut monitor = Monitor::new(battery(21));
        assert_eq!(monitor.update(battery(21)), None);
        assert_eq!(monitor.update(battery(20)), Some(20));
        for percent in [20, 19, 21, 24, 20, 0] {
            assert_eq!(monitor.update(battery(percent)), None);
        }
    }

    #[test] fn recharging_or_recovery_rearms_warning() {
        let mut monitor = Monitor::new(battery(10));
        assert_eq!(monitor.update(battery(10)), Some(10));
        assert_eq!(monitor.update(Snapshot::from_raw(1, 8, 10)), None);
        assert_eq!(monitor.update(battery(10)), Some(10));
        assert_eq!(monitor.update(battery(25)), None);
        assert_eq!(monitor.update(battery(20)), Some(20));
    }

    #[test] fn unknown_and_absent_readings_never_warn_or_repeat_warning() {
        let mut monitor = Monitor::new(battery(10));
        assert_eq!(monitor.update(Snapshot::from_raw(0, 128, 5)), None);
        assert_eq!(monitor.update(Snapshot::from_raw(0, 255, 5)), None);
        assert_eq!(monitor.update(battery(10)), Some(10));
        assert_eq!(monitor.update(Snapshot::from_raw(255, 255, 255)), None);
        assert_eq!(monitor.update(Snapshot::from_raw(255, 255, 80)), None);
        assert_eq!(monitor.update(Snapshot::from_raw(0, 255, 80)), None);
        assert_eq!(monitor.update(battery(10)), None);
    }
}
