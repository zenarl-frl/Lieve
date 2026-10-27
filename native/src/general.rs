use super::*;
impl App {
    pub(super) unsafe fn toggle_fullscreen_hide(&mut self,hwnd:HWND) {
        self.fullscreen_auto_hide=!self.fullscreen_auto_hide;self.save();self.refresh_visibility(hwnd,true);InvalidateRect(hwnd,null(),0);
    }
    pub(super) unsafe fn toggle_autostart(&mut self,hwnd:HWND) {
        match autostart::set(!self.autostart_enabled) {
            Ok(())=>self.autostart_enabled=!self.autostart_enabled,
            Err(error)=>{MessageBoxW(hwnd,wide(&format!("Could not update Windows startup: {error}")).as_ptr(),wide("Lieve").as_ptr(),MB_OK|MB_ICONERROR);}
        }InvalidateRect(hwnd,null(),0);
    }
    pub(super) unsafe fn refresh_visibility(&mut self,hwnd:HWND,force:bool) {
        let now=std::time::Instant::now();
        if !force && self.desktop_check.elapsed()<std::time::Duration::from_millis(500) {return;}
        self.desktop_check=now;
        if self.menu_open || self.system_drag.is_some() || self.dragging_glow {return;}
        let hide=self.fullscreen_auto_hide && desktop::fullscreen_over(hwnd);
        if hide==self.hidden_fullscreen {return;}self.hidden_fullscreen=hide;
        if hide {
            ShowWindow(hwnd,SW_HIDE);self.clock.stop(hwnd);
            self.navigation.blur();
            for id in [3,4,5,6,8,9] {KillTimer(hwnd,id);}
            self.transition=None;self.expanded=self.pinned;self.tracking=false;self.hover=layout::Hover::None;
            self.osd=osd::Hud::new();self.spectrum.enable(false);self.spectrum_visual=spectrum::Visual::new();
            self.resize(hwnd);
        }else {
            // Resize while hidden so no stale OSD or half-open frame flashes on return.
            self.resize(hwnd);ShowWindow(hwnd,SW_SHOWNOACTIVATE);self.refresh_gaze(hwnd,now);
            SetTimer(hwnd,4,self.blink.delay_ms(),None);if self.devil_grin {SetTimer(hwnd,6,12000,None);}
        }
    }
    pub(super) unsafe fn general_click(&mut self,hwnd:HWND,x:i32,y:i32) {
        if (16..128).contains(&x) && (88..112).contains(&y) {self.tab=6;self.resize(hwnd);}
        else if (264..384).contains(&x) && (88..112).contains(&y) {self.tab=11;self.resize(hwnd);}
        else if (16..384).contains(&x) && (124..166).contains(&y) {self.toggle_autostart(hwnd);}
        else if (16..384).contains(&x) && (176..218).contains(&y) {self.toggle_fullscreen_hide(hwnd);}
    }
    pub(super) unsafe fn general_view(&self,dc:HDC) {
        let card=rgb(18,18,25);let white=rgb(239,239,247);let muted=rgb(143,143,164);
        self.framed_tile(dc,layout::Rect{x:16,y:88,w:112,h:24},10,card,rgb(35,35,47));
        self.fitted_text(dc,"Appearance",layout::Rect{x:16,y:88,w:112,h:24},self.small_font,white,DT_CENTER);
        self.fitted_text(dc,"System",layout::Rect{x:144,y:88,w:104,h:24},self.font,white,DT_LEFT);
        self.framed_tile(dc,layout::Rect{x:264,y:88,w:120,h:24},10,card,rgb(35,35,47));
        self.fitted_text(dc,"Spectrum",layout::Rect{x:264,y:88,w:120,h:24},self.small_font,white,DT_CENTER);
        for (y,label,hint,on) in [(124,"Start with Windows","Launch this executable at sign-in",self.autostart_enabled),
            (176,"Hide in fullscreen","Hide for games / video on this monitor",self.fullscreen_auto_hide)] {
            self.framed_tile(dc,layout::Rect{x:16,y,w:368,h:42},14,card,rgb(35,35,47));
            self.fitted_text(dc,label,layout::Rect{x:28,y:y+4,w:300,h:18},self.small_font,white,DT_LEFT);
            self.fitted_text(dc,hint,layout::Rect{x:28,y:y+23,w:300,h:14},self.caption_font,muted,DT_LEFT);
            self.toggle_switch(dc,layout::Rect{x:336,y:y+12,w:32,h:18},on,card);
        }
        self.fitted_text(dc,if self.hotkey_registered {"Ctrl+Alt+Space open / focus  ·  Esc close"}else{"Ctrl+Alt+Space unavailable (already in use)"},layout::Rect{x:16,y:226,w:368,h:14},self.caption_font,muted,DT_CENTER);
        self.fitted_text(dc,"Tab / arrows navigate  ·  Enter select",layout::Rect{x:16,y:244,w:368,h:12},self.caption_font,muted,DT_CENTER);
    }
}
