use crate::rgb;
pub const TEXT:u32=rgb(239,239,247);
pub const MUTED:u32=rgb(143,143,164);
pub const CARD:u32=rgb(18,18,25);
pub const LIFT:u32=rgb(26,26,36);
pub const BORDER:u32=rgb(35,35,47);
pub const RAIL:u32=rgb(43,43,57);
pub const BACKGROUND:u32=rgb(8,8,11);

// Keep white toggle thumbs/check marks legible even with very light eye colours.
pub fn control_accent(color:u32)->u32 {
    let (r,g,b)=(color&255,(color>>8)&255,(color>>16)&255);
    let light=(299*r+587*g+114*b)/1000;
    let channel=|v|if light>110 {v*110/light}else if light<72 {v+(255-v)*(72-light)/(255-light)}else{v};
    rgb(channel(r),channel(g),channel(b))
}
