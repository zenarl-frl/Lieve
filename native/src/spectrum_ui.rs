use super::*;
pub const SENSITIVITY:layout::Rect=layout::Rect{x:28,y:219,w:344,h:22};
impl App {
    pub(super) unsafe fn spectrum_settings_changed(&mut self,hwnd:HWND) {
        self.save();self.refresh_spectrum(hwnd);InvalidateRect(hwnd,null(),0);
    }
    pub(super) unsafe fn spectrum_click(&mut self,hwnd:HWND,x:i32,y:i32) {
        if (16..128).contains(&x) && (88..112).contains(&y) {self.tab=10;self.resize(hwnd);}
        else if (16..384).contains(&x) && (120..156).contains(&y) {self.spectrum_preferences.enabled=!self.spectrum_preferences.enabled;self.spectrum_settings_changed(hwnd);}
        else if (168..192).contains(&y) && (220..372).contains(&x) {
            self.spectrum_preferences.white=x>=300;self.spectrum_settings_changed(hwnd);
        }else if SENSITIVITY.contains(x,y) {
            self.system_drag=Some(system_ui::Drag::Sensitivity);SetCapture(hwnd);self.system_slider(hwnd,system_ui::Drag::Sensitivity,x);
        }
    }
    pub(super) unsafe fn spectrum_view(&self,dc:HDC) {
        let white=rgb(239,239,247);let muted=rgb(143,143,164);let card=rgb(18,18,25);
        self.framed_tile(dc,layout::Rect{x:16,y:88,w:112,h:24},10,card,rgb(35,35,47));
        self.fitted_text(dc,"System",layout::Rect{x:16,y:88,w:112,h:24},self.small_font,white,DT_CENTER);
        self.fitted_text(dc,"Spectrum settings",layout::Rect{x:144,y:88,w:240,h:24},self.font,white,DT_LEFT);
        self.framed_tile(dc,layout::Rect{x:16,y:120,w:368,h:36},14,card,rgb(35,35,47));
        self.fitted_text(dc,"Live spectrum",layout::Rect{x:28,y:128,w:280,h:20},self.small_font,white,DT_LEFT);
        let enabled=self.spectrum_preferences.enabled;
        self.toggle_switch(dc,layout::Rect{x:336,y:129,w:32,h:18},enabled,card);
        self.fitted_text(dc,"Bar colour",layout::Rect{x:28,y:170,w:160,h:20},self.small_font,white,DT_LEFT);
        for (x,label,selected) in [(220,"Accent",!self.spectrum_preferences.white),(300,"White",self.spectrum_preferences.white)] {
            self.framed_tile(dc,layout::Rect{x,y:168,w:72,h:24},10,if selected {rgb(39,30,43)}else{card},if selected {self.color}else{rgb(35,35,47)});
            self.fitted_text(dc,label,layout::Rect{x,y:168,w:72,h:24},self.small_font,white,DT_CENTER);
        }
        self.fitted_text(dc,"Sensitivity",layout::Rect{x:28,y:200,w:240,h:18},self.small_font,white,DT_LEFT);
        let value=self.spectrum_preferences.sensitivity;
        self.fitted_text(dc,&format!("{value}%"),layout::Rect{x:292,y:200,w:80,h:18},self.small_font,muted,DT_RIGHT);
        self.tile(dc,28,228,344,5,5,rgb(44,44,57));let width=((value-50)*344/150) as i32;
        if width>0 {self.tile(dc,28,228,width,5,5,self.color);}
        self.smooth_tile(dc,layout::Rect{x:22+width,y:224,w:13,h:13},13,white,rgb(8,8,11));
        self.fitted_text(dc,"5 bars  ·  Real Windows output audio",layout::Rect{x:28,y:242,w:344,h:12},self.caption_font,muted,DT_CENTER);
    }
}
