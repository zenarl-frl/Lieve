use std::ptr::null_mut;
use windows_sys::Win32::Graphics::Gdi::*;

// Grow-only 32-bit DIB buffers avoid allocating a DC/bitmap at every shell size.
pub struct Surface {pub dc:HDC,bitmap:HBITMAP,original:HGDIOBJ,width:i32,height:i32,pixels:*mut u8}
impl Surface {
    pub fn new()->Self {Self {dc:null_mut(),bitmap:null_mut(),original:null_mut(),width:0,height:0,pixels:null_mut()}}
    pub unsafe fn ensure(&mut self,reference:HDC,width:i32,height:i32)->bool {
        if !self.dc.is_null() && self.width>=width && self.height>=height {return true;}
        let width=((width.max(self.width).max(1)+63)/64)*64;
        let height=((height.max(self.height).max(1)+63)/64)*64;
        let mut info=std::mem::zeroed::<BITMAPINFO>();
        info.bmiHeader=BITMAPINFOHEADER{biSize:std::mem::size_of::<BITMAPINFOHEADER>() as u32,biWidth:width,biHeight:-height,biPlanes:1,biBitCount:32,biCompression:BI_RGB,..std::mem::zeroed()};
        let mut pixels=std::ptr::null_mut();
        let image=CreateDIBSection(reference,&info,DIB_RGB_COLORS,&mut pixels,null_mut(),0);
        if image.is_null() {return false;}
        if self.dc.is_null() {self.dc=CreateCompatibleDC(reference);}
        if self.dc.is_null() {DeleteObject(image);return false;}
        let previous=SelectObject(self.dc,image);
        if self.bitmap.is_null() {self.original=previous;}else{DeleteObject(self.bitmap);}
        self.bitmap=image; self.width=width; self.height=height; self.pixels=pixels.cast();true
    }
    pub unsafe fn resolve(&self,output:&mut Raster,width:i32,height:i32,factor:i32) {
        if self.pixels.is_null() || width<=0 || height<=0 || factor<=0 || width*factor>self.width || height*factor>self.height {return;}
        // GDI batches writes to DIB sections; flush before CPU coverage resolve.
        GdiFlush();
        let source=std::slice::from_raw_parts(self.pixels,self.width as usize*self.height as usize*4);
        output.resize(width as usize,height as usize);
        resolve(source,self.width as usize, &mut output.pixels,width as usize,height as usize,factor as usize);
    }
    pub unsafe fn copy_pixels(&self,output:&mut Raster,width:i32,height:i32) {
        GdiFlush();output.resize(width as usize,height as usize);
        let source=std::slice::from_raw_parts(self.pixels,self.width as usize*self.height as usize*4);
        for y in 0..height as usize {output.pixels[y*width as usize*4..(y+1)*width as usize*4].copy_from_slice(&source[y*self.width as usize*4..y*self.width as usize*4+width as usize*4]);}
    }
}
impl Drop for Surface {
    fn drop(&mut self) {unsafe {
        if !self.dc.is_null() {SelectObject(self.dc,self.original);DeleteObject(self.bitmap);DeleteDC(self.dc);}
    }}
}
pub struct Surfaces {pub output:Surface,pub layers:[Surface;4]}
impl Surfaces {pub fn new()->Self {Self {output:Surface::new(),layers:std::array::from_fn(|_|Surface::new())}}}

fn resolve(source:&[u8],stride:usize,target:&mut [u8],width:usize,height:usize,factor:usize) {
    let samples=(factor*factor) as u32;
    for y in 0..height {for x in 0..width {
        let mut sum=[0u32;3];
        for sy in 0..factor {for sx in 0..factor {
            let at=((y*factor+sy)*stride+x*factor+sx)*4;
            for c in 0..3 {sum[c]+=source[at+c] as u32;}
        }}
        let at=(y*width+x)*4;for c in 0..3 {target[at+c]=((sum[c]+samples/2)/samples) as u8;}target[at+3]=255;
    }}
}

pub struct Raster {pub width:usize,pub height:usize,pub pixels:Vec<u8>}
impl Raster {
    pub fn new()->Self {Self {width:0,height:0,pixels:Vec::new()}}
    fn resize(&mut self,w:usize,h:usize) {self.width=w;self.height=h;self.pixels.resize(w*h*4,0);}
    pub unsafe fn draw(&self,dc:HDC,x:i32,y:i32) {
        if self.width==0 || self.height==0 {return;}
        let mut info=std::mem::zeroed::<BITMAPINFO>();
        info.bmiHeader=BITMAPINFOHEADER{biSize:std::mem::size_of::<BITMAPINFOHEADER>() as u32,biWidth:self.width as i32,biHeight:-(self.height as i32),biPlanes:1,biBitCount:32,biCompression:BI_RGB,..std::mem::zeroed()};
        // One-to-one copy: filtering/coverage already resolved in RGB, no HALFTONE.
        SetStretchBltMode(dc,COLORONCOLOR);
        StretchDIBits(dc,x,y,self.width as i32,self.height as i32,0,0,self.width as i32,self.height as i32,self.pixels.as_ptr().cast(),&info,DIB_RGB_COLORS,SRCCOPY);
    }
    pub unsafe fn upload_layered(&self,hwnd:windows_sys::Win32::Foundation::HWND,surface:&mut Surface)->bool {
        use windows_sys::Win32::{Foundation::{POINT,SIZE},UI::WindowsAndMessaging::*};
        let screen=GetDC(std::ptr::null_mut());
        let mut success=false;
        if surface.ensure(screen,self.width as i32,self.height as i32) {
            // Copy premultiplied pixels directly; GDI raster operations can erase alpha.
            for y in 0..self.height {std::ptr::copy_nonoverlapping(self.pixels.as_ptr().add(y*self.width*4),surface.pixels.add(y*surface.width as usize*4),self.width*4);}
            let size=SIZE{cx:self.width as i32,cy:self.height as i32};let origin=POINT{x:0,y:0};
            success=UpdateLayeredWindow(hwnd,screen,std::ptr::null(),&size,surface.dc,&origin,0,&BLENDFUNCTION{BlendOp:AC_SRC_OVER as u8,BlendFlags:0,SourceConstantAlpha:255,AlphaFormat:AC_SRC_ALPHA as u8},ULW_ALPHA)!=0;
        }ReleaseDC(std::ptr::null_mut(),screen);success
    }
    pub fn rounded_rect(&mut self,width:usize,height:usize,radius:f64,fill:u32,border:u32,background:u32) {
        self.resize(width,height);
        let inside=|x:f64,y:f64,w:f64,h:f64,r:f64| {
            if x<0. || y<0. || x>=w || y>=h || w<=0. || h<=0. {return false;}
            let r=r.max(0.).min(w.min(h)/2.);
            let dx=if x<r {r-x}else if x>w-r {x-(w-r)}else{0.};
            let dy=if y<r {r-y}else if y>h-r {y-(h-r)}else{0.};dx*dx+dy*dy<=r*r
        };
        for y in 0..height {for x in 0..width {
            let (mut outer,mut inner)=(0,0);
            for sy in 0..4 {for sx in 0..4 {
                let (px,py)=(x as f64+(sx as f64+0.5)/4.,y as f64+(sy as f64+0.5)/4.);
                if inside(px,py,width as f64,height as f64,radius) {outer+=1;}
                if inside(px-1.,py-1.,width as f64-2.,height as f64-2.,radius-1.) {inner+=1;}
            }}
            let at=(y*width+x)*4;
            for c in 0..3 {let shift=if c==0 {16}else if c==1 {8}else{0};
                let channel=|color:u32|((color>>shift)&255) as u32;
                self.pixels[at+c]=((channel(fill)*inner+channel(border)*(outer-inner)+channel(background)*(16-outer)+8)/16) as u8;
            }self.pixels[at+3]=255;
        }}
    }
    pub fn thumbnail(&mut self,image:&crate::artwork::Image,size:usize,radius:f64,background:u32) {
        self.resize(size,size);
        let side=image.width.min(image.height) as f64;
        let (ox,oy)=((image.width as f64-side)/2.,(image.height as f64-side)/2.);
        let ratio=side/size as f64;
        for y in 0..size {for x in 0..size {
            let mut color=[0.;3];
            if ratio>=1. {
                let (x0,x1,y0,y1)=(ox+x as f64*ratio,ox+(x+1) as f64*ratio,oy+y as f64*ratio,oy+(y+1) as f64*ratio);
                for sy in y0.floor() as usize..y1.ceil() as usize {for sx in x0.floor() as usize..x1.ceil() as usize {
                    let weight=(x1.min(sx as f64+1.)-x0.max(sx as f64)).max(0.)*(y1.min(sy as f64+1.)-y0.max(sy as f64)).max(0.)/(ratio*ratio);
                    let at=(sy.min(image.height as usize-1)*image.width as usize+sx.min(image.width as usize-1))*4;
                    for c in 0..3 {color[c]+=image.bgra[at+c] as f64*weight;}
                }}
            } else {
                let px=(ox+(x as f64+0.5)*ratio-0.5).clamp(0.,image.width as f64-1.);
                let py=(oy+(y as f64+0.5)*ratio-0.5).clamp(0.,image.height as f64-1.);
                let (ix,iy)=(px.floor() as usize,py.floor() as usize);let (fx,fy)=(px-ix as f64,py-iy as f64);
                for (sx,wx) in [(ix,1.-fx),((ix+1).min(image.width as usize-1),fx)] {
                    for (sy,wy) in [(iy,1.-fy),((iy+1).min(image.height as usize-1),fy)] {
                        let at=(sy*image.width as usize+sx)*4;for c in 0..3 {color[c]+=image.bgra[at+c] as f64*wx*wy;}
                    }
                }
            }
            let r=radius.min(size as f64/2.).max(0.);
            let mut coverage=1.;
            if r>0. && ((x as f64)<r || x as f64>=size as f64-r) && ((y as f64)<r || y as f64>=size as f64-r) {
                let (cx,cy)=(if (x as f64)<r {r}else{size as f64-r},if (y as f64)<r {r}else{size as f64-r});
                let mut inside=0;
                for sy in 0..4 {for sx in 0..4 {let dx=x as f64+(sx as f64+0.5)/4.-cx;let dy=y as f64+(sy as f64+0.5)/4.-cy;if dx*dx+dy*dy<=r*r {inside+=1;}}}
                coverage=inside as f64/16.;
            }
            let at=(y*size+x)*4;
            for c in 0..3 {let shift=if c==0 {16}else if c==1 {8}else{0};let bg=((background>>shift)&255) as f64;
                self.pixels[at+c]=(color[c]*coverage+bg*(1.-coverage)).round().clamp(0.,255.) as u8;}
            self.pixels[at+3]=255;
        }}
    }
}
pub struct ShapeCache {entries:Vec<((usize,usize,i32,u32,u32,u32),Raster)>}
impl ShapeCache {
    pub fn new()->Self {Self {entries:Vec::new()}}
    pub unsafe fn draw(&mut self,dc:HDC,x:i32,y:i32,width:usize,height:usize,diameter:i32,fill:u32,border:u32,background:u32) {
        let key=(width,height,diameter,fill,border,background);
        let index=if let Some(index)=self.entries.iter().position(|entry|entry.0==key) {index}else {
            if self.entries.len()>=32 {self.entries.clear();}
            let mut raster=Raster::new();raster.rounded_rect(width,height,diameter as f64/2.,fill,border,background);
            self.entries.push((key,raster));self.entries.len()-1
        };
        self.entries[index].1.draw(dc,x,y);
    }
}
pub struct CoverCache {image:Option<std::sync::Arc<crate::artwork::Image>>,size:usize,bg:u32,raster:Raster}
impl CoverCache {
    pub fn new()->Self {Self {image:None,size:0,bg:0,raster:Raster::new()}}
    pub unsafe fn draw(&mut self,dc:HDC,image:&std::sync::Arc<crate::artwork::Image>,x:i32,y:i32,size:usize,bg:u32,radius:f64) {
        if self.size!=size || self.bg!=bg || !self.image.as_ref().is_some_and(|old|std::sync::Arc::ptr_eq(old,image)) {
            self.raster.thumbnail(image,size,radius,bg);self.image=Some(image.clone());self.size=size;self.bg=bg;
        }
        self.raster.draw(dc,x,y);
    }
}

#[cfg(test)] mod tests {
    use super::*;
    use windows_sys::Win32::System::Threading::{GetCurrentProcess,GetGuiResources,GR_GDIOBJECTS};
    #[test] fn coverage_resolve_and_thumbnail_filter_average_instead_of_dithering() {
        let source=[0,0,0,255,255,255,255,255,255,255,255,255,0,0,0,255];let mut out=[0;4];
        resolve(&source,2,&mut out,1,1,2);assert_eq!(out,[128,128,128,255]);
        let image=crate::artwork::Image{width:2,height:2,bgra:source.to_vec()};let mut raster=Raster::new();
        raster.thumbnail(&image,1,0.,0);assert_eq!(raster.pixels,[128,128,128,255]);
        raster.thumbnail(&crate::artwork::Image{width:1,height:1,bgra:vec![255;4]},8,4.,0);
        assert!(raster.pixels[0]<255);assert_eq!(raster.pixels[(4*8+4)*4],255);
        raster.rounded_rect(8,8,4.,0xffffff,0xffffff,0);
        assert!(raster.pixels.chunks_exact(4).any(|p|p[0]>0 && p[0]<255));
    }
    #[test] fn buffers_reuse_capacity_and_release_replaced_and_final_gdi_objects() {unsafe {
        let screen=GetDC(null_mut()); assert!(!screen.is_null());
        let mut warmup=Surface::new(); assert!(warmup.ensure(screen,80,60)); drop(warmup); GdiFlush();
        let before=GetGuiResources(GetCurrentProcess(),GR_GDIOBJECTS);
        for _ in 0..32 {
            let mut surface=Surface::new(); assert!(surface.ensure(screen,80,60));
            let first=surface.bitmap;
            assert!(surface.ensure(screen,81,61)); assert_eq!(surface.bitmap,first);
            assert!(surface.ensure(screen,260,160)); assert_ne!(surface.bitmap,first);
        }
        GdiFlush();
        assert_eq!(GetGuiResources(GetCurrentProcess(),GR_GDIOBJECTS),before);
        ReleaseDC(null_mut(),screen);
    }}
}
