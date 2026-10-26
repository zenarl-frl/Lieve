use crate::render::{Surface,Raster};
use windows_sys::Win32::Foundation::{POINT,RECT};
use windows_sys::Win32::Graphics::Gdi::*;

#[derive(Clone,Copy)]
pub enum Glyph {Play,Pause,Previous,Next,Lyrics,Settings,Control,Volume,Muted,Brightness,Wifi,Bluetooth}
pub struct Renderer {surface:Surface,raster:Raster}
impl Renderer {
    pub fn new()->Self {Self {surface:Surface::new(),raster:Raster::new()}}
    pub unsafe fn draw(&mut self,target:HDC,scale:f32,x:i32,y:i32,size:i32,glyph:Glyph,color:u32,background:u32,disc:Option<u32>) {
        let pixels=(size as f32*scale).round() as i32;let high=pixels*4;
        if !self.surface.ensure(target,high,high) {return;}
        let dc=self.surface.dc;
        let fill=CreateSolidBrush(background);FillRect(dc,&RECT{left:0,top:0,right:high,bottom:high},fill);DeleteObject(fill);
        let old_pen=SelectObject(dc,GetStockObject(NULL_PEN));
        let old_brush=SelectObject(dc,GetStockObject(NULL_BRUSH));
        if let Some(disc)=disc {
            let brush=CreateSolidBrush(disc);SelectObject(dc,brush);Ellipse(dc,0,0,high,high);
            SelectObject(dc,old_brush);DeleteObject(brush);
        }
        let factor=if disc.is_some() {0.56}else{1.};
        let p=|n:f64|((12.+(n-12.)*factor)*high as f64/24.).round() as i32;
        let point=|x:f64,y:f64|POINT{x:p(x),y:p(y)};
        let brush=CreateSolidBrush(color);SelectObject(dc,brush);
        match glyph {
            Glyph::Play=> {let points=[point(9.,5.),point(20.,12.),point(9.,19.)];Polygon(dc,points.as_ptr(),3);}
            Glyph::Pause=> {for x in [7.,14.] {Rectangle(dc,p(x),p(5.),p(x+3.),p(19.));}}
            Glyph::Previous|Glyph::Next=> {
                let mx=|x:f64|if matches!(glyph,Glyph::Next) {24.-x}else{x};
                let (a,b)=(mx(5.),mx(7.));Rectangle(dc,p(a.min(b)),p(6.),p(a.max(b)),p(18.));
                for x in [8.,14.] {let points=[point(mx(x),12.),point(mx(x+6.),6.),point(mx(x+6.),18.)];Polygon(dc,points.as_ptr(),3);}
            }
            Glyph::Lyrics|Glyph::Settings|Glyph::Control|Glyph::Volume|Glyph::Muted|Glyph::Brightness|Glyph::Wifi|Glyph::Bluetooth=> {
                SelectObject(dc,GetStockObject(NULL_BRUSH));
                let pen=CreatePen(PS_SOLID,(high as f64/24.*1.4).round().max(1.) as i32,color);SelectObject(dc,pen);
                if matches!(glyph,Glyph::Lyrics) {
                    RoundRect(dc,p(3.),p(4.),p(21.),p(17.),p(4.)-p(0.),p(4.)-p(0.));
                    MoveToEx(dc,p(7.),p(17.),std::ptr::null_mut());LineTo(dc,p(6.),p(21.));LineTo(dc,p(12.),p(17.));
                    for x in [8.,14.] {MoveToEx(dc,p(x),p(8.),std::ptr::null_mut());LineTo(dc,p(x+2.),p(8.));LineTo(dc,p(x+2.),p(11.));LineTo(dc,p(x),p(13.));}
                } else if matches!(glyph,Glyph::Settings|Glyph::Brightness) {
                    if matches!(glyph,Glyph::Brightness) {Ellipse(dc,p(8.),p(8.),p(16.),p(16.));}
                    else {
                    Ellipse(dc,p(7.),p(7.),p(17.),p(17.));Ellipse(dc,p(10.),p(10.),p(14.),p(14.));
                    }
                    for i in 0..8 {let (sin,cos)=(i as f64*std::f64::consts::FRAC_PI_4).sin_cos();
                        MoveToEx(dc,p(12.+cos*6.),p(12.+sin*6.),std::ptr::null_mut());LineTo(dc,p(12.+cos*9.),p(12.+sin*9.));}
                } else if matches!(glyph,Glyph::Control) {
                    for (y,cx) in [(7.,8.),(17.,16.)] {
                        MoveToEx(dc,p(3.),p(y),std::ptr::null_mut());LineTo(dc,p(21.),p(y));
                        let b=CreateSolidBrush(background);let previous=SelectObject(dc,b);Ellipse(dc,p(cx-3.),p(y-3.),p(cx+3.),p(y+3.));SelectObject(dc,previous);DeleteObject(b);
                    }
                } else if matches!(glyph,Glyph::Wifi) {
                    for coords in [[(3.,8.),(8.,2.),(16.,2.),(21.,8.)],[(6.,12.),(10.,8.),(14.,8.),(18.,12.)],[(9.,16.),(11.,14.),(13.,14.),(15.,16.)]] {
                        let points=coords.map(|(x,y)|point(x,y));PolyBezier(dc,points.as_ptr(),4);
                    }Ellipse(dc,p(11.),p(19.),p(13.),p(21.));
                } else if matches!(glyph,Glyph::Bluetooth) {
                    MoveToEx(dc,p(12.),p(3.),std::ptr::null_mut());LineTo(dc,p(12.),p(21.));LineTo(dc,p(18.),p(16.));LineTo(dc,p(6.),p(8.));
                    MoveToEx(dc,p(12.),p(3.),std::ptr::null_mut());LineTo(dc,p(18.),p(8.));LineTo(dc,p(6.),p(16.));
                } else {
                    let points=[point(3.,9.),point(8.,9.),point(13.,5.),point(13.,19.),point(8.,15.),point(3.,15.)];Polygon(dc,points.as_ptr(),6);
                    if matches!(glyph,Glyph::Muted) {MoveToEx(dc,p(17.),p(9.),std::ptr::null_mut());LineTo(dc,p(22.),p(16.));MoveToEx(dc,p(22.),p(9.),std::ptr::null_mut());LineTo(dc,p(17.),p(16.));}
                    else {let points=[point(16.,7.),point(22.,10.),point(22.,14.),point(16.,17.)];PolyBezier(dc,points.as_ptr(),4);}
                }
                SelectObject(dc,GetStockObject(NULL_PEN));DeleteObject(pen);
            }
        }
        SelectObject(dc,old_brush);SelectObject(dc,old_pen);DeleteObject(brush);
        self.surface.resolve(&mut self.raster,pixels,pixels,4);
        self.raster.draw(target,(x as f32*scale).round() as i32,(y as f32*scale).round() as i32);
    }
}
