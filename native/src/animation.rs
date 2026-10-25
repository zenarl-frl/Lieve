use std::time::Instant;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Frame { pub x: i32, pub y: i32, pub w: i32, pub h: i32 }

pub struct Transition {
    pub from: Frame,
    pub to: Frame,
    pub started: Instant,
}

pub fn interpolate(from: Frame, to: Frame, progress: f64) -> Frame {
    let t = progress.clamp(0., 1.);
    // Quintic ease-out: fast response with a gentle, zero-velocity landing.
    let ease = 1. - (1. - t).powi(5);
    let lerp = |a: i32, b: i32| (a as f64 + (b-a) as f64 * ease).round() as i32;
    Frame { x:lerp(from.x,to.x), y:lerp(from.y,to.y), w:lerp(from.w,to.w), h:lerp(from.h,to.h) }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn exact_endpoints() {
        let a=Frame{x:0,y:0,w:144,h:36}; let b=Frame{x:100,y:0,w:470,h:322};
        assert_eq!(interpolate(a,b,0.),a); assert_eq!(interpolate(a,b,1.),b);
        assert_eq!(interpolate(a,b,2.),b);
    }
    #[test] fn no_overshoot_and_monotonic_size() {
        let a=Frame{x:0,y:0,w:144,h:36}; let b=Frame{x:100,y:0,w:470,h:322};
        let mut previous=a;
        for i in 0..=100 { let f=interpolate(a,b,i as f64/100.);
            assert!((previous.w..=b.w).contains(&f.w));
            assert!((previous.h..=b.h).contains(&f.h)); previous=f;
        }
    }
}
