use windows_sys::Win32::{Foundation::*,System::LibraryLoader::*,UI::WindowsAndMessaging::*};

#[repr(C)] struct Accent {state:i32,flags:u32,color:u32,animation:u32}
#[repr(C)] struct Attribute {kind:u32,data:*mut core::ffi::c_void,size:usize}
pub struct Glass {pub active:bool,pub blur_available:bool,pub failed:std::cell::Cell<bool>,frosted:bool}
impl Glass {
    pub fn new()->Self {Self{active:false,blur_available:false,failed:std::cell::Cell::new(false),frosted:false}}
    pub unsafe fn configure(&mut self,hwnd:HWND,active:bool,frosted:bool)->bool {
        if self.active==active && self.frosted==frosted {return false;}
        self.failed.set(false);
        let style=GetWindowLongPtrW(hwnd,GWL_EXSTYLE) as u32;
        SetWindowLongPtrW(hwnd,GWL_EXSTYLE,if active {style|WS_EX_LAYERED}else{style&!WS_EX_LAYERED} as isize);
        self.blur_available=false;
        let user=GetModuleHandleW(crate::wide("user32.dll").as_ptr());
        if let Some(function)=GetProcAddress(user,b"SetWindowCompositionAttribute\0".as_ptr()) {
            let apply:unsafe extern "system" fn(HWND,*mut Attribute)->i32=std::mem::transmute(function);
            let mut accent=Accent{state:if active && frosted {3}else{0},flags:0,color:0,animation:0};
            let mut data=Attribute{kind:19,data:(&mut accent as *mut Accent).cast(),size:std::mem::size_of::<Accent>()};
            self.blur_available=apply(hwnd,&mut data)!=0 && active && frosted;
        }
        self.active=active;self.frosted=frosted;true
    }
}
pub fn opacity(transparency:u32)->u8 {(((100-transparency.min(100))*255+50)/100).max(1) as u8}
pub struct Opaque {pub rect:RECT,pub radius:i32,pub strength:f64}
fn coverage(x:i32,y:i32,w:i32,h:i32,radius:i32,floating:bool)->f64 {
    let r=radius.min(w/2).min(h/2).max(0) as f64;
    if r<=0. || (!floating && (y as f64)<r) || ((x as f64)>=r && (x as f64)<w as f64-r)
        || ((y as f64)>=r && (y as f64)<h as f64-r) {return 1.;}
    let cx=if x<w/2 {r}else{w as f64-r};let cy=if y<h/2 {r}else{h as f64-r};let mut inside=0;
    for sy in 0..4 {for sx in 0..4 {let dx=x as f64+(sx as f64+0.5)/4.-cx;let dy=y as f64+(sy as f64+0.5)/4.-cy;if dx*dx+dy*dy<=r*r {inside+=1;}}}
    inside as f64/16.
}
pub unsafe fn present(hwnd:HWND,source:&crate::render::Surface,output:&mut crate::render::Raster,upload:&mut crate::render::Surface,width:i32,height:i32,transparency:u32,radius:i32,floating:bool,opaque:&[Opaque],position:Option<(i32,i32)>)->bool {
    source.copy_pixels(output,width,height);let base=opacity(transparency) as u32;
    for y in 0..height {for x in 0..width {
        let at=(y as usize*width as usize+x as usize)*4;
        let color=&mut output.pixels[at..at+4];let brightest=color[0].max(color[1]).max(color[2]) as u32;
        let mut alpha=if brightest>=90 {255}else{(base+brightest.saturating_sub(8)*2).min(255)};
        for area in opaque {let r=&area.rect;if x>=r.left && x<r.right && y>=r.top && y<r.bottom {
            let amount=coverage(x-r.left,y-r.top,r.right-r.left,r.bottom-r.top,area.radius,true)*area.strength.clamp(0.,1.);
            alpha=(alpha as f64+(255-alpha) as f64*amount).round() as u32;
        }}
        let alpha=(alpha as f64*coverage(x,y,width,height,radius,floating)).round() as u32;
        for c in 0..3 {color[c]=((color[c] as u32*alpha+127)/255) as u8;}color[3]=alpha as u8;
    }}
    output.upload_layered(hwnd,upload,position)
}
#[cfg(test)] mod tests {
    use super::*;
    #[test] fn glass_opacity_and_corner_coverage_are_bounded() {
        assert_eq!(opacity(0),255);assert_eq!(opacity(100),1);assert_eq!(opacity(999),1);
        assert_eq!(coverage(0,0,100,100,20,false),1.);assert_eq!(coverage(0,0,100,100,20,true),0.);
        assert_eq!(coverage(50,50,100,100,20,true),1.);
    }
}
