use std::time::Instant;

// Match prototype/src/components/Notch.tsx and TabBar.tsx.
#[derive(Clone, Copy)]
pub struct Spring { pub stiffness: f64, pub damping: f64, pub mass: f64 }
pub const SHELL: Spring = Spring { stiffness:360., damping:32., mass:0.85 };
pub const TAB: Spring = Spring { stiffness:480., damping:38., mass:1. };

pub fn spring(from:f64, to:f64, velocity:f64, seconds:f64, spec:Spring)->(f64,f64) {
    let t=seconds.max(0.);
    let decay=spec.damping/(2.*spec.mass);
    let frequency=(spec.stiffness/spec.mass-decay*decay).sqrt();
    let displacement=from-to;
    let b=(velocity+decay*displacement)/frequency;
    let (sin,cos)=(frequency*t).sin_cos();
    let wave=displacement*cos+b*sin;
    let envelope=(-decay*t).exp();
    (to+envelope*wave,envelope*((b*cos-displacement*sin)*frequency-decay*wave))
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Frame { pub x:i32, pub y:i32, pub w:i32, pub h:i32, pub radius:i32 }
impl Frame {
    fn values(self)->[f64;5] { [self.x as f64,self.y as f64,self.w as f64,self.h as f64,self.radius as f64] }
}

pub struct Transition { pub from:Frame, pub to:Frame, pub velocity:[f64;5], pub started:Instant }
impl Transition {
    pub fn sample(&self, now:Instant)->(Frame,[f64;5],bool) {
        let t=now.saturating_duration_since(self.started).as_secs_f64();
        let from=self.from.values(); let to=self.to.values();
        let mut position=[0.;5]; let mut velocity=[0.;5]; let mut settled=true;
        for i in 0..5 {
            (position[i],velocity[i])=spring(from[i],to[i],self.velocity[i],t,SHELL);
            settled &= (position[i]-to[i]).abs()<0.25 && velocity[i].abs()<1.;
        }
        if settled || t>=1.2 { return (self.to,[0.;5],true); }
        (Frame {x:position[0].round() as i32,y:position[1].round() as i32,
            w:(position[2].round() as i32).max(1),h:(position[3].round() as i32).max(1),
            radius:(position[4].round() as i32).max(0)},velocity,false)
    }
}

pub struct Pill { from:f64, pub to:f64, velocity:f64, started:Instant }
impl Pill {
    pub fn new()->Self { let x=super::layout::TAB_START as f64; Self {from:x,to:x,velocity:0.,started:Instant::now()} }
    pub fn sample(&self,now:Instant)->(f64,bool) {
        let (x,v)=self.state(now);
        let settled=(x-self.to).abs()<0.1 && v.abs()<1.;
        (if settled {self.to}else{x},settled)
    }
    fn state(&self,now:Instant)->(f64,f64) {
        spring(self.from,self.to,self.velocity,now.saturating_duration_since(self.started).as_secs_f64(),TAB)
    }
    pub fn retarget(&mut self,to:f64,now:Instant,motion:bool) {
        if to==self.to && motion {return;}
        let (x,v)=self.state(now);
        self.from=if motion {x}else{to}; self.velocity=if motion {v}else{0.}; self.to=to; self.started=now;
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stage { Idle=0, Compact=1, Panel=2,Osd=3 }
fn duration(stage:Stage)->f64 { match stage {Stage::Idle=>0.18,Stage::Compact|Stage::Osd=>0.14,Stage::Panel=>0.16} }
pub struct Content { pub stage:Stage, from:[f64;4], started:Instant, exit:f64, moving:bool }
impl Content {
    pub fn new()->Self { Self {stage:Stage::Idle,from:[1.,0.,0.,0.],started:Instant::now(),exit:0.,moving:false} }
    pub fn retarget(&mut self,stage:Stage,now:Instant,motion:bool) {
        if self.stage==stage && motion {return;}
        let weights=self.sample(now).0;
        self.exit=duration(self.stage)*weights.iter().copied().fold(0.,f64::max);
        self.from=weights; self.stage=stage; self.started=now; self.moving=motion;
    }
    pub fn sample(&self,now:Instant)->([f64;4],bool) {
        let mut weights=[0.;4];
        let elapsed=now.saturating_duration_since(self.started).as_secs_f64();
        if !self.moving || elapsed>=self.exit+duration(self.stage) {
            weights[self.stage as usize]=1.; return (weights,true);
        }
        // AnimatePresence mode="wait": outgoing content exits before incoming enters.
        if elapsed<self.exit {
            let fade=1.-smooth(elapsed/self.exit);
            for i in 0..4 {weights[i]=self.from[i]*fade;}
        } else { weights[self.stage as usize]=smooth((elapsed-self.exit)/duration(self.stage)); }
        (weights,false)
    }
}
fn smooth(t:f64)->f64 { let t=t.clamp(0.,1.); t*t*(3.-2.*t) }

pub struct Blink { started:Option<Instant>, cycle:usize }
impl Blink {
    pub fn new()->Self {Self {started:None,cycle:0}}
    pub fn start(&mut self,now:Instant) {self.started=Some(now); self.cycle=self.cycle.wrapping_add(1);}
    pub fn delay_ms(&self)->u32 {[4200,6700,3500,7800][self.cycle%4]}
    pub fn sample(&self,now:Instant)->(f64,bool) {
        let Some(start)=self.started else {return (1.,true);};
        let t=now.saturating_duration_since(start).as_secs_f64();
        // Web mascot: 100 ms close, 120 ms blink state, 500 ms recovery.
        if t>=0.62 {return (1.,true);}
        let ease=|t:f64|1.-(1.-t.clamp(0.,1.)).powi(5);
        if t<0.1 {(1.-0.94*ease(t/0.1),false)}
        else if t<0.12 {(0.06,false)}
        else {(0.06+0.94*ease((t-0.12)/0.5),false)}
    }
}

#[cfg(test)] mod tests {
    use super::*;
    use std::time::Duration;
    #[test] fn spring_starts_at_current_position_and_velocity() {
        let (x,v)=spring(144.,470.,73.,0.,SHELL);
        assert!((x-144.).abs()<1e-9); assert!((v-73.).abs()<1e-9);
        let (x,v)=spring(144.,470.,0.,1.2,SHELL);
        assert!((x-470.).abs()<0.001); assert!(v.abs()<0.001);
    }
    #[test] fn shell_response_matches_reference_spring() {
        // Analytic reference for k=360, c=32, m=0.85, initial velocity zero.
        let (x,_)=spring(0.,1.,0.,0.1,SHELL);
        assert!((x-0.643).abs()<0.003);
        for i in 0..120 { let (x,v)=spring(0.,1.,0.,i as f64/100.,SHELL);
            assert!(x.is_finite() && v.is_finite()); assert!((-0.001..=1.002).contains(&x));
        }
    }
    #[test] fn reversal_preserves_momentum_and_settles_at_new_target() {
        let (x,v)=spring(144.,470.,0.,0.1,SHELL);
        let (next,next_v)=spring(x,144.,v,0.,SHELL);
        assert_eq!(next,x); assert!((next_v-v).abs()<1e-9);
        let (end,_)=spring(x,144.,v,1.2,SHELL); assert!((end-144.).abs()<0.001);
    }
    #[test] fn frame_has_exact_settled_target_and_keeps_docked_center() {
        let now=Instant::now();
        let transition=Transition {from:Frame{x:428,y:0,w:144,h:36,radius:20},
            to:Frame{x:300,y:0,w:400,h:220,radius:22},velocity:[0.;5],started:now};
        for i in 0..120 { let (f,_,_)=transition.sample(now+Duration::from_millis(i*10));
            assert!((f.x*2+f.w-1000).abs()<=1); assert_eq!(f.y,0); assert!(f.w>0 && f.h>0);
        }
        let (f,v,done)=transition.sample(now+Duration::from_secs(2));
        assert_eq!(f,transition.to); assert_eq!(v,[0.;5]); assert!(done);
    }
    #[test] fn content_waits_then_fades_without_a_jump_on_reversal() {
        let now=Instant::now(); let mut content=Content::new();
        content.retarget(Stage::Panel,now,true);
        let (weights,_)=content.sample(now+Duration::from_millis(90));
        assert!(weights[0]>0. && weights[0]<1.); assert_eq!(weights[2],0.);
        let time=now+Duration::from_millis(240);
        let before=content.sample(time).0; assert!(before[2]>0. && before[2]<1.);
        content.retarget(Stage::Idle,time,true); assert_eq!(content.sample(time).0,before);
        assert_eq!(content.sample(time+Duration::from_secs(1)),([1.,0.,0.,0.],true));
        content.retarget(Stage::Panel,time,false);
        assert_eq!(content.sample(time),([0.,0.,1.,0.],true));
    }
    #[test] fn tab_pill_retargets_without_jumping_and_reduced_motion_snaps() {
        let now=Instant::now(); let mut pill=Pill::new();
        pill.retarget(386.,now,true);
        let halfway=now+Duration::from_millis(70);
        let (x,done)=pill.sample(halfway); assert!(x>16. && x<386. && !done);
        pill.retarget(90.,halfway,true);
        assert!((pill.sample(halfway).0-x).abs()<1e-9);
        assert!(pill.sample(halfway+Duration::from_secs(2)).1);
        pill.retarget(238.,halfway,false); assert_eq!(pill.sample(halfway),(238.,true));
    }
    #[test] fn blink_closes_holds_recovers_and_stops_requesting_frames() {
        let now=Instant::now(); let mut blink=Blink::new();
        assert_eq!(blink.sample(now),(1.,true));
        blink.start(now); assert_eq!(blink.sample(now),(1.,false));
        assert_eq!(blink.sample(now+Duration::from_millis(110)),(0.06,false));
        let (scale,done)=blink.sample(now+Duration::from_millis(250));
        assert!(scale>0.06 && scale<1. && !done);
        assert_eq!(blink.sample(now+Duration::from_millis(620)),(1.,true));
    }
}
