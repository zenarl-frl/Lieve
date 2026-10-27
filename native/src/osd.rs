use std::time::{Instant,Duration};
#[derive(Clone,Copy,Debug,PartialEq,Eq)] pub enum Kind {Volume,Brightness,PowerConnected,PowerDisconnected,LowBattery}
impl Kind {pub fn is_power(self)->bool {matches!(self,Self::PowerConnected|Self::PowerDisconnected|Self::LowBattery)}}
pub struct Hud {pub kind:Kind,pub muted:bool,pub power_level:Option<u8>,pub charging:bool,pub preview:bool,from:f64,target:f64,velocity:f64,started:Instant,shown_at:Instant,motion:bool,until:Option<Instant>}
impl Hud {
    pub fn new()->Self {let now=Instant::now();Self{kind:Kind::Volume,muted:false,power_level:None,charging:false,preview:false,from:0.,target:0.,velocity:0.,started:now,shown_at:now,motion:true,until:None}}
    pub fn show(&mut self,kind:Kind,value:u8,muted:bool,now:Instant,motion:bool) {
        let (current,velocity,_)=self.state(now);let visible=self.visible(now);let same=self.kind==kind && visible;
        if !visible {self.shown_at=now;}self.motion=motion;
        self.power_level=None;self.charging=false;self.preview=false;
        self.from=if motion {if same {current}else{0.}}else{value as f64};
        self.velocity=if motion && same {velocity}else{0.};self.target=value.min(100) as f64;
        self.kind=kind;self.muted=muted;self.started=now;self.until=Some(now+Duration::from_secs(2));
    }
    pub fn show_power(&mut self,kind:Kind,level:Option<u8>,charging:bool,preview:bool,now:Instant,motion:bool) {
        self.show(kind,level.unwrap_or(0),false,now,motion);self.power_level=level.filter(|p|*p<=100);self.charging=charging;self.preview=preview;
        self.until=Some(now+Duration::from_secs(3));
    }
    pub fn sweep(&self,now:Instant)->Option<f64> {
        (self.motion && self.charging && self.kind==Kind::PowerConnected && self.visible(now))
            .then(||(now.saturating_duration_since(self.shown_at).as_secs_f64()/0.9).fract())
    }
    pub fn visible(&self,now:Instant)->bool {self.until.is_some_and(|end|now<end)}
    pub fn expiry_delay(&self,now:Instant)->Option<u32> {
        self.until.filter(|end|*end>now).map(|end|end.duration_since(now).as_nanos().div_ceil(1_000_000).clamp(1,u32::MAX as u128) as u32)
    }
    pub fn opacity(&self,now:Instant)->f64 {
        let Some(end)=self.until else{return 0.;};
        if !self.motion {return if now<end {1.}else{0.};}
        if now>=end {(1.-now.duration_since(end).as_secs_f64()/0.18).clamp(0.,1.)}
        else {(now.saturating_duration_since(self.shown_at).as_secs_f64()/0.12).clamp(0.,1.)}
    }
    pub fn value(&self,now:Instant)->f64 {self.state(now).0.clamp(0.,100.)}
    fn state(&self,now:Instant)->(f64,f64,bool) {
        let t=now.saturating_duration_since(self.started).as_secs_f64();
        let (v,speed)=crate::animation::spring(self.from,self.target,self.velocity,t,crate::animation::SHELL);
        let done=t>=1. || ((v-self.target).abs()<0.1 && speed.abs()<1.);
        if done {(self.target,0.,true)}else{(v,speed,false)}
    }
    pub fn moving(&self,now:Instant)->bool {
        self.until.is_some_and(|end| (now<end && (!self.state(now).2 || self.opacity(now)<1. || self.sweep(now).is_some())) || (now>=end && self.opacity(now)>0.))
    }
}
#[cfg(test)] mod tests {
    use super::*;
    #[test] fn osd_retargets_from_current_value_and_extends_expiry() {
        let now=Instant::now();let mut hud=Hud::new();assert!(!hud.visible(now));
        hud.show(Kind::Volume,80,false,now,true);let at=now+Duration::from_millis(100);let value=hud.value(at);
        assert!(value>0. && value<80.);hud.show(Kind::Volume,20,true,at,true);assert!((hud.value(at)-value).abs()<1e-9);
        assert!(hud.visible(now+Duration::from_millis(2050)));assert!(!hud.visible(at+Duration::from_secs(2)));
        assert_eq!(hud.expiry_delay(at+Duration::from_millis(1999)),Some(1));
        assert_eq!(hud.expiry_delay(at+Duration::from_secs(2)),None);
        assert!(!hud.moving(at+Duration::from_secs(3)));
        hud.show(Kind::Brightness,50,false,now,false);assert_eq!(hud.value(now),50.);
    }
    #[test] fn repeated_keys_keep_osd_visible_and_reduced_motion_is_immediate() {
        let now=Instant::now();let mut hud=Hud::new();hud.show(Kind::Volume,30,false,now,true);
        let at=now+Duration::from_millis(150);assert_eq!(hud.opacity(at),1.);
        hud.show(Kind::Volume,40,false,at,true);assert_eq!(hud.opacity(at),1.);
        hud.show(Kind::Brightness,60,false,at,true);assert_eq!(hud.opacity(at),1.);
        hud.show(Kind::Brightness,70,false,at,false);assert_eq!(hud.opacity(at),1.);
        assert_eq!(hud.opacity(at+Duration::from_secs(2)),0.);
    }
    #[test] fn power_hud_has_bounded_animation_and_keeps_unknown_percent_unknown() {
        let now=Instant::now();let mut hud=Hud::new();hud.show_power(Kind::PowerConnected,None,true,false,now,true);
        assert_eq!(hud.power_level,None);assert!(hud.moving(now+Duration::from_secs(2)));assert!(hud.sweep(now+Duration::from_secs(1)).is_some());
        assert!(!hud.visible(now+Duration::from_secs(3)));assert!(!hud.moving(now+Duration::from_secs(4)));
        hud.show_power(Kind::PowerDisconnected,Some(77),false,true,now,false);assert_eq!(hud.power_level,Some(77));assert!(hud.preview);assert!(hud.sweep(now).is_none());assert_eq!(hud.opacity(now),1.);
    }
}
