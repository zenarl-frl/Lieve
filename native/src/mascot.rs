use std::time::Instant;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Pose { pub dx:f64, pub dy:f64, pub tilt:f64, pub lid:f64 }
impl Default for Pose {
    fn default()->Self {Self {dx:0.,dy:0.,tilt:0.,lid:1.}}
}
impl Pose {
    pub fn from_cursor(cursor:(i32,i32),bounds:(i32,i32,i32,i32),idle_ms:u32)->Self {
        if idle_ms>=26_000 {return Self {tilt:3.5,lid:0.26,..Self::default()};}
        let (left,top,right,bottom)=bounds;
        if right<=left || bottom<=top {return Self::default();}
        let x=((cursor.0 as f64-left as f64)/(right as f64-left as f64)*2.-1.).clamp(-1.,1.);
        let y=((cursor.1 as f64-top as f64)/(bottom as f64-top as f64)*2.-1.).clamp(-1.,1.);
        // Mascot.tsx uses SVG-space translate(1.6*x,1.1*y) and rotate(1.4*x).
        let scale=68./160.;
        Self {dx:x*1.6*scale,dy:y*1.1*scale,tilt:x*1.4,lid:1.}
    }
    pub fn point(self,point:(f64,f64),blink:f64)->(f64,f64) {
        // Blink's minimum is 0.06, including when the sleepy lid is already low.
        let squash=0.06+(self.lid-0.06)*((blink-0.06)/0.94).clamp(0.,1.);
        let (sin,cos)=self.tilt.to_radians().sin_cos();
        let (x,y)=(point.0-34.,(point.1-11.)*squash);
        (34.+self.dx+x*cos-y*sin,11.+self.dy+x*sin+y*cos)
    }
}

// GetLastInputInfo and GetTickCount share a wrapping 32-bit millisecond clock.
pub fn idle_millis(now:u32,last_input:u32)->u32 {now.wrapping_sub(last_input)}

pub struct Gaze { from:Pose, pub target:Pose, velocity:[f64;4], started:Instant }
impl Gaze {
    pub fn new()->Self {Self {from:Pose::default(),target:Pose::default(),velocity:[0.;4],started:Instant::now()}}
    pub fn retarget(&mut self,target:Pose,now:Instant,motion:bool)->bool {
        if target==self.target && motion {return false;}
        let (pose,velocity,_)=self.state(now);
        self.from=if motion {pose}else{target}; self.velocity=if motion {velocity}else{[0.;4]};
        self.target=target; self.started=now;
        self.from!=target || self.velocity.iter().any(|v|v.abs()>0.01)
    }
    pub fn sample(&self,now:Instant)->(Pose,bool) {
        let (pose,_,settled)=self.state(now); (pose,settled)
    }
    fn state(&self,now:Instant)->(Pose,[f64;4],bool) {
        let elapsed=now.saturating_duration_since(self.started).as_secs_f64();
        let from=[self.from.dx,self.from.dy,self.from.tilt,self.from.lid];
        let to=[self.target.dx,self.target.dy,self.target.tilt,self.target.lid];
        let mut position=[0.;4]; let mut velocity=[0.;4]; let mut settled=true;
        for i in 0..4 {
            // Critically damped tracking preserves velocity across live targets,
            // rather than restarting a zero-velocity CSS-style tween each poll.
            let rate=if i==3 {14.}else{12.};
            let d=from[i]-to[i]; let b=self.velocity[i]+rate*d;
            let envelope=(-rate*elapsed).exp();
            position[i]=to[i]+(d+b*elapsed)*envelope;
            velocity[i]=(self.velocity[i]-rate*b*elapsed)*envelope;
            settled &= (position[i]-to[i]).abs()<0.001 && velocity[i].abs()<0.01;
        }
        if settled || elapsed>=1.2 {return (self.target,[0.;4],true);}
        (Pose {dx:position[0],dy:position[1],tilt:position[2],lid:position[3]},velocity,false)
    }
}

pub struct Grin {started:Option<Instant>}
impl Grin {
    pub fn new()->Self {Self {started:None}}
    pub fn start(&mut self,now:Instant) {self.started=Some(now);}
    pub fn reset(&mut self) {self.started=None;}
    pub fn sample(&self,now:Instant)->(f64,bool) {
        let Some(start)=self.started else {return (0.,true);};
        let t=now.saturating_duration_since(start).as_secs_f64();
        let ease=|v:f64|{let v=v.clamp(0.,1.);v*v*(3.-2.*v)};
        if t>=1.9 {(0.,true)}else if t<0.32 {(ease(t/0.32),false)}
        else if t<1.35 {(1.,false)}else{(1.-ease((t-1.35)/0.55),false)}
    }
}

pub unsafe fn draw_grin(dc:windows_sys::Win32::Graphics::Gdi::HDC,scale:f64,amount:f64) {
    use windows_sys::Win32::{Foundation::POINT,Graphics::Gdi::*};
    if amount<0.005 {return;}
    let p=|x:f64,y:f64|POINT{x:((12.+34.+(x-34.)*(0.6+0.4*amount))*scale).round() as i32,
        y:((12.+24.+(y-24.)*amount-9.*amount)*scale).round() as i32};
    let curve=[p(12.,24.),p(24.,29.),p(44.,29.),p(56.,24.),p(52.,40.),p(16.,40.),p(12.,24.)];
    let shade=|value:u32| {let channel=|s:u32|(8.+(((value>>s)&255) as f64-8.)*amount).round() as u32;
        crate::rgb(channel(0),channel(8),channel(16))};
    SaveDC(dc);SelectObject(dc,GetStockObject(NULL_PEN));
    let inside=CreateSolidBrush(crate::rgb(16,5,12));let old=SelectObject(dc,inside);
    BeginPath(dc);PolyBezier(dc,curve.as_ptr(),7);CloseFigure(dc);EndPath(dc);FillPath(dc);
    SelectObject(dc,old);DeleteObject(inside);
    BeginPath(dc);PolyBezier(dc,curve.as_ptr(),7);CloseFigure(dc);EndPath(dc);SelectClipPath(dc,RGN_AND);
    let teeth=CreateSolidBrush(shade(crate::rgb(240,238,248)));let previous=SelectObject(dc,teeth);    for (x,length) in [(18.,7.),(26.,5.),(34.,6.),(42.,5.),(50.,7.)] {
        let u=(x-34.)/22.;let y=24.+4.*(1.-u*u);
        let points=[p(x-3.,y-1.),p(x+3.,y-1.),p(x,y+length)];Polygon(dc,points.as_ptr(),3);
    }
    for x in [22.,30.,38.,46.] {
        let u=(x-34.)/22.;let y=36.-12.*u*u;
        let points=[p(x-3.,y+1.),p(x+3.,y+1.),p(x,y-3.)];Polygon(dc,points.as_ptr(),3);
    }
    SelectObject(dc,previous);DeleteObject(teeth);RestoreDC(dc,-1);
    // The grin stays a fixed near-white expression, independent of the eye colour.
    let pen=CreatePen(PS_SOLID,scale.round().max(1.) as i32,shade(crate::rgb(214,214,224)));let old=SelectObject(dc,pen);
    BeginPath(dc);PolyBezier(dc,curve.as_ptr(),7);CloseFigure(dc);EndPath(dc);StrokePath(dc);
    SelectObject(dc,old);DeleteObject(pen);
}

#[cfg(test)] mod tests {
    use super::*;
    use std::time::Duration;
    #[test] fn grin_opens_holds_and_returns_to_rest_with_bounded_animation() {
        let now=Instant::now();let mut grin=Grin::new();assert_eq!(grin.sample(now),(0.,true));
        grin.start(now);assert_eq!(grin.sample(now),(0.,false));
        assert!(grin.sample(now+Duration::from_millis(160)).0>0.4);
        assert_eq!(grin.sample(now+Duration::from_millis(700)),(1.,false));
        assert_eq!(grin.sample(now+Duration::from_millis(1900)),(0.,true));
        grin.reset();assert_eq!(grin.sample(now),(0.,true));
    }
    #[test] fn cursor_mapping_handles_negative_monitors_and_clamps_other_screens() {
        let bounds=(-1920,-200,0,880);
        assert_eq!(Pose::from_cursor((-960,340),bounds,0),Pose::default());
        let edge=Pose::from_cursor((0,880),bounds,0);
        assert!((edge.dx-0.68).abs()<1e-9); assert_eq!(edge.tilt,1.4);
        assert_eq!(Pose::from_cursor((2000,2000),bounds,0),edge);
        assert_eq!(Pose::from_cursor((0,0),(0,0,0,0),0),Pose::default());
    }
    #[test] fn sleepy_threshold_and_clock_wrap_are_correct() {
        let bounds=(0,0,100,100);
        assert_eq!(Pose::from_cursor((50,50),bounds,25_999).lid,1.);
        let sleepy=Pose::from_cursor((100,100),bounds,26_000);
        assert_eq!(sleepy.dx,0.); assert_eq!(sleepy.dy,0.); assert_eq!(sleepy.tilt,3.5); assert_eq!(sleepy.lid,0.26);
        assert_eq!(idle_millis(25,u32::MAX-24),50);
    }
    #[test] fn interruptions_keep_pose_continuous_and_reduced_motion_is_neutral() {
        let now=Instant::now(); let mut gaze=Gaze::new();
        let target=Pose::from_cursor((100,100),(0,0,100,100),0);
        assert!(gaze.retarget(target,now,true));
        let midway=now+Duration::from_millis(150); let before=gaze.sample(midway).0;
        assert!(before.dx>0. && before.dx<target.dx);
        assert!(!gaze.retarget(target,midway,true));
        gaze.retarget(Pose::default(),midway,true);
        assert_eq!(gaze.sample(midway).0,before);
        assert_eq!(gaze.sample(midway+Duration::from_millis(1200)),(Pose::default(),true));
        gaze.retarget(target,midway,true);
        gaze.retarget(Pose::default(),midway,false);
        assert_eq!(gaze.sample(midway),(Pose::default(),true));
    }
    #[test] fn moving_targets_preserve_velocity_and_do_not_restart_the_frame_response() {
        let now=Instant::now(); let mut gaze=Gaze::new();
        let a=Pose::from_cursor((100,50),(0,0,100,100),0);
        gaze.retarget(a,now,true);
        let at=now+Duration::from_millis(100); let (position,velocity,_)=gaze.state(at);
        gaze.retarget(Pose::from_cursor((0,50),(0,0,100,100),0),at,true);
        let (next,next_velocity,_)=gaze.state(at);
        assert!((position.dx-next.dx).abs()<1e-12 && (position.dy-next.dy).abs()<1e-12);
        assert!((position.tilt-next.tilt).abs()<1e-12 && (position.lid-next.lid).abs()<1e-12);
        for i in 0..4 {assert!((velocity[i]-next_velocity[i]).abs()<1e-9);}
        assert!(gaze.sample(at+Duration::from_millis(1)).0.dx>position.dx);
        assert!(gaze.sample(at+Duration::from_secs(2)).1);
    }
    #[test] fn sleepy_blink_keeps_a_fixed_center_and_recovers_to_sleepy_lid() {
        let sleepy=Pose {lid:0.26,..Pose::default()};
        assert_eq!(sleepy.point((34.,11.),0.06),(34.,11.));
        let (_,closed)=sleepy.point((34.,21.),0.06);
        let (_,open)=sleepy.point((34.,21.),1.);
        assert!((closed-11.6).abs()<1e-9); assert!((open-13.6).abs()<1e-9);
        let (_,awake)=Pose::default().point((34.,21.),1.); assert_eq!(awake,21.);
    }
}
