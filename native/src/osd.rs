use std::time::{Instant,Duration};
#[derive(Clone,Copy,Debug,PartialEq,Eq)] pub enum Kind {Volume,Brightness}
pub struct Hud {pub kind:Kind,pub muted:bool,from:f64,target:f64,velocity:f64,started:Instant,until:Option<Instant>}
impl Hud {
    pub fn new()->Self {Self{kind:Kind::Volume,muted:false,from:0.,target:0.,velocity:0.,started:Instant::now(),until:None}}
    pub fn show(&mut self,kind:Kind,value:u8,muted:bool,now:Instant,motion:bool) {
        let (current,velocity,_)=self.state(now);let same=self.kind==kind && self.until.is_some();
        self.from=if motion {if same {current}else{0.}}else{value as f64};
        self.velocity=if motion && same {velocity}else{0.};self.target=value.min(100) as f64;
        self.kind=kind;self.muted=muted;self.started=now;self.until=Some(now+Duration::from_secs(2));
    }
    pub fn visible(&self,now:Instant)->bool {self.until.is_some_and(|end|now<end)}
    pub fn expiry_delay(&self,now:Instant)->Option<u32> {
        self.until.filter(|end|*end>now).map(|end|end.duration_since(now).as_nanos().div_ceil(1_000_000).clamp(1,u32::MAX as u128) as u32)
    }
    pub fn opacity(&self,now:Instant)->f64 {
        let Some(end)=self.until else{return 0.;};
        if now>=end {(1.-now.duration_since(end).as_secs_f64()/0.18).clamp(0.,1.)}
        else {(now.saturating_duration_since(self.started).as_secs_f64()/0.12).clamp(0.,1.)}
    }
    pub fn value(&self,now:Instant)->f64 {self.state(now).0.clamp(0.,100.)}
    fn state(&self,now:Instant)->(f64,f64,bool) {
        let t=now.saturating_duration_since(self.started).as_secs_f64();
        let (v,speed)=crate::animation::spring(self.from,self.target,self.velocity,t,crate::animation::SHELL);
        let done=t>=1. || ((v-self.target).abs()<0.1 && speed.abs()<1.);
        if done {(self.target,0.,true)}else{(v,speed,false)}
    }
    pub fn moving(&self,now:Instant)->bool {
        self.until.is_some_and(|end| (now<end && (!self.state(now).2 || self.opacity(now)<1.)) || (now>=end && self.opacity(now)>0.))
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
}
