use std::time::{Duration,Instant,SystemTime,UNIX_EPOCH};

#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum Focus {Paused(u32),Running(u64)}
fn millis(now:SystemTime)->u64 {now.duration_since(UNIX_EPOCH).unwrap_or_default().as_millis().min(u64::MAX as u128) as u64}
impl Focus {
    pub fn capture(left:u32,end:Option<Instant>,now:Instant,wall:SystemTime)->Self {
        match end {Some(end)=>Self::Running(millis(wall).saturating_add(end.saturating_duration_since(now).as_millis().min(u64::MAX as u128) as u64)),None=>Self::Paused(left)}
    }
    pub fn argument(self)->String {match self {Self::Paused(left)=>format!("--lieve-focus=p:{left}"),Self::Running(end)=>format!("--lieve-focus=r:{end}")}}
    pub fn parse(argument:&str)->Option<Self> {
        let value=argument.strip_prefix("--lieve-focus=")?;
        if let Some(left)=value.strip_prefix("p:") {let left=left.parse::<u32>().ok()?;return (left<=86400).then_some(Self::Paused(left));}
        value.strip_prefix("r:")?.parse().ok().map(Self::Running)
    }
    pub fn restore(self,now:Instant,wall:SystemTime)->(u32,Option<Instant>) {
        match self {
            Self::Paused(left)=>(left,None),
            Self::Running(end)=> {
                let remaining=end.saturating_sub(millis(wall)).min(86_400_000);
                (remaining.div_ceil(1000) as u32,(remaining>0).then_some(now+Duration::from_millis(remaining)))
            }
        }
    }
}
pub fn launch(focus:Focus)->std::io::Result<()> {
    std::process::Command::new(std::env::current_exe()?).arg(focus.argument()).spawn()?;Ok(())
}

#[cfg(test)] mod tests {
    use super::*;
    #[test] fn restart_preserves_running_deadline_across_startup_and_paused_state() {
        let now=Instant::now();let wall=UNIX_EPOCH+Duration::from_secs(1_000_000);
        let state=Focus::capture(99,Some(now+Duration::from_secs(90)),now,wall);
        let parsed=Focus::parse(&state.argument()).unwrap();
        let (left,end)=parsed.restore(now+Duration::from_secs(4),wall+Duration::from_secs(4));
        assert_eq!(left,86);assert_eq!(end,Some(now+Duration::from_secs(90)));
        assert_eq!(Focus::parse(&Focus::Paused(87).argument()).unwrap().restore(now,wall),(87,None));
        assert_eq!(state.restore(now+Duration::from_secs(100),wall+Duration::from_secs(100)),(0,None));
        assert_eq!(Focus::parse("--lieve-focus=p:999999"),None);assert_eq!(Focus::parse("--lieve-focus=nope"),None);
    }
}
