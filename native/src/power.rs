use windows_sys::Win32::System::Power::{GetSystemPowerStatus, SYSTEM_POWER_STATUS};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Source { Ac, Battery, Unknown }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Snapshot {
    pub source: Source,
    pub present: Option<bool>,
    pub percent: Option<u8>,
}

impl Snapshot {
    fn from_raw(ac: u8, flags: u8, percent: u8) -> Self {
        let present = if flags == 255 { None } else { Some(flags & 128 == 0) };
        Self {
            source: match ac { 0 => Source::Battery, 1 => Source::Ac, _ => Source::Unknown },
            present,
            percent: if present == Some(false) { None } else { Some(percent).filter(|p| *p <= 100) },
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
}

impl Monitor {
    pub fn new(snapshot: Snapshot) -> Self { Self { snapshot, warned: false } }

    // Rearm only on AC or after recovery to 25%, not on noisy 20/21% readings.
    // Unknown readings retain the latch and never generate an alert.
    pub fn update(&mut self, snapshot: Snapshot) -> Option<u8> {
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

#[cfg(test)]
mod tests {
    use super::*;
    fn battery(percent: u8) -> Snapshot { Snapshot::from_raw(0, 0, percent) }

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
