use super::*;
#[derive(Clone,Copy,PartialEq,Eq)] pub enum Drag {Volume,Brightness,Transparency}
pub const VOLUME:layout::Rect=layout::Rect{x:76,y:168,w:252,h:24};
pub const BRIGHTNESS:layout::Rect=layout::Rect{x:76,y:216,w:252,h:24};
pub const TRANSPARENCY:layout::Rect=layout::Rect{x:28,y:194,w:344,h:24};
pub fn percent(x:i32,rect:layout::Rect)->u8 {((x-rect.x).clamp(0,rect.w)*100/rect.w) as u8}
impl App {
    pub(super) unsafe fn open_windows_settings(&mut self,hwnd:HWND,uri:&str) {
        ShellExecuteW(hwnd,wide("open").as_ptr(),wide(uri).as_ptr(),null(),null(),SW_SHOWNORMAL);
    }
    pub(super) unsafe fn toggle_control(&mut self,hwnd:HWND) {
        if matches!(self.tab,7|9) {self.tab=self.control_return;}else{self.control_return=self.tab;self.tab=7;}
        self.system_controller.refresh();self.resize(hwnd);
    }
    pub(super) unsafe fn show_osd(&mut self,hwnd:HWND,kind:osd::Kind,value:u8,muted:bool) {
        let now=std::time::Instant::now();self.osd.show(kind,value,muted,now,self.motion_enabled);
        SetTimer(hwnd,8,2000,None);self.resize(hwnd);self.clock.start(hwnd);
    }
    pub(super) unsafe fn system_slider(&mut self,hwnd:HWND,kind:Drag,x:i32) {
        let rect=match kind {Drag::Volume=>VOLUME,Drag::Brightness=>BRIGHTNESS,Drag::Transparency=>TRANSPARENCY};
        let value=percent(x,rect);
        match kind {
            Drag::Volume=>if self.system.audio.volume.is_some() && self.system.audio.volume!=Some(value) {
                self.system_controller.volume(value);if self.system.audio.muted {self.system_controller.mute(false);}
                self.system.audio.volume=Some(value);self.system.audio.muted=false;self.show_osd(hwnd,osd::Kind::Volume,value,false);
            },
            Drag::Brightness=>if self.system.devices.brightness.is_some() && self.system.devices.brightness!=Some(value) {
                self.system_controller.brightness(value);self.system.devices.brightness=Some(value);self.show_osd(hwnd,osd::Kind::Brightness,value,false);
            },
            Drag::Transparency=>if self.transparency!=value as u32 {self.transparency=value as u32;self.resize(hwnd);},
        }
        InvalidateRect(hwnd,null(),0);
    }
    unsafe fn start_system_drag(&mut self,hwnd:HWND,kind:Drag,x:i32) {
        self.system_drag=Some(kind);SetCapture(hwnd);self.system_slider(hwnd,kind,x);
    }
    pub(super) unsafe fn control_click(&mut self,hwnd:HWND,x:i32,y:i32) {
        if self.tab==9 {
            if (88..112).contains(&y) && x<128 {self.tab=7;self.resize(hwnd);}
            else if (230..254).contains(&y) {self.open_windows_settings(hwnd,"ms-settings:bluetooth");}
            return;
        }
        if (88..148).contains(&y) && (16..196).contains(&x) {self.open_windows_settings(hwnd,"ms-settings:network-wifi");}
        else if (88..148).contains(&y) && (204..384).contains(&x) {self.tab=9;self.system_controller.refresh();self.resize(hwnd);}
        else if (24..64).contains(&x) && (156..196).contains(&y) && self.system.audio.volume.is_some() {
            let muted=!self.system.audio.muted;self.system_controller.mute(muted);self.system.audio.muted=muted;
            self.show_osd(hwnd,osd::Kind::Volume,self.system.audio.volume.unwrap_or(0),muted);
        } else if VOLUME.contains(x,y) && self.system.audio.volume.is_some() {self.start_system_drag(hwnd,Drag::Volume,x);}
        else if BRIGHTNESS.contains(x,y) && self.system.devices.brightness.is_some() {self.start_system_drag(hwnd,Drag::Brightness,x);}
        else if (204..244).contains(&y) {self.open_windows_settings(hwnd,"ms-settings:display");}
    }
    pub(super) unsafe fn effects_click(&mut self,hwnd:HWND,x:i32,y:i32) {
        if (16..128).contains(&x) && (88..112).contains(&y) {self.tab=6;self.resize(hwnd);}
        else if (28..376).contains(&x) && (126..166).contains(&y) {self.frosted=!self.frosted;self.resize(hwnd);self.save();}
        else if TRANSPARENCY.contains(x,y) {self.start_system_drag(hwnd,Drag::Transparency,x);}
    }
    pub(super) unsafe fn system_updates(&mut self,hwnd:HWND) {
        let mut changed=false;
        while let Ok(update)=self.system_updates.try_recv() {
            match update {
                system_control::Update::Audio(audio)=> {
                    let previous=self.system.audio.clone();
                    if self.system_drag!=Some(Drag::Volume) {self.system.audio=audio.clone();
                        if let (Some(before),Some(value))=(previous.volume,audio.volume) {if before!=value || previous.muted!=audio.muted {self.show_osd(hwnd,osd::Kind::Volume,value,audio.muted);}}
                    }
                }
                system_control::Update::Devices(mut devices)=> {
                    devices.brightness=self.system.devices.brightness;self.system.devices=devices;
                }
                system_control::Update::Brightness(value)=>if self.system_drag!=Some(Drag::Brightness) {
                    let previous=self.system.devices.brightness;self.system.devices.brightness=value;
                    if let (Some(before),Some(value))=(previous,value) {if before!=value {self.show_osd(hwnd,osd::Kind::Brightness,value,false);}}
                },
                system_control::Update::Error(error)=> {self.system.error=Some((error,std::time::Instant::now()));}
            }changed=true;
        }
        if self.system.error.as_ref().is_some_and(|(_,time)|time.elapsed()>std::time::Duration::from_secs(5)) {self.system.error=None;changed=true;}
        if !self.expanded && self.content.stage==animation::Stage::Osd && !self.osd.visible(std::time::Instant::now()) {self.resize(hwnd);}
        if !self.expanded && self.glass.active && self.transition.is_none() && self.content.sample(std::time::Instant::now()).1 && self.tab_pill.sample(std::time::Instant::now()).1
            && !self.osd.visible(std::time::Instant::now()) {self.resize(hwnd);}
        if changed && self.expanded {InvalidateRect(hwnd,null(),0);}
    }
    pub(super) unsafe fn control_view(&self,dc:HDC) {
        let white=rgb(239,239,247);let muted=rgb(143,143,164);let card=rgb(18,18,25);
        if self.tab==9 {
            self.framed_tile(dc,layout::Rect{x:16,y:88,w:100,h:24},10,card,rgb(35,35,47));
            self.fitted_text(dc,"Back",layout::Rect{x:16,y:88,w:100,h:24},self.small_font,white,DT_CENTER);
            self.fitted_text(dc,"Bluetooth devices",layout::Rect{x:132,y:88,w:252,h:24},self.small_font,white,DT_LEFT);
            if self.system.devices.connected.is_empty() {
                self.fitted_text(dc,&self.system.devices.bluetooth,layout::Rect{x:24,y:132,w:352,h:52},self.font,muted,DT_CENTER);
            }
            for (row,name) in self.system.devices.connected.iter().take(4).enumerate() {
                let y=120+row as i32*28;self.framed_tile(dc,layout::Rect{x:16,y,w:368,h:24},10,card,rgb(35,35,47));
                self.fitted_text(dc,name,layout::Rect{x:28,y,w:344,h:24},self.small_font,white,DT_LEFT);
            }
            self.fitted_text(dc,"Open Bluetooth settings",layout::Rect{x:16,y:230,w:368,h:24},self.small_font,self.color,DT_CENTER);return;
        }
        for x in [16,204] {self.framed_tile(dc,layout::Rect{x,y:88,w:180,h:60},18,card,rgb(35,35,47));}
        let wifi=if self.system.devices.wifi_connected {rgb(95,155,255)}else{muted};
        self.icon(dc,28,103,30,icons::Glyph::Wifi,wifi,card,None);
        self.fitted_text(dc,"Wi-Fi",layout::Rect{x:72,y:94,w:112,h:18},self.small_font,white,DT_LEFT);
        self.fitted_text(dc,&self.system.devices.wifi,layout::Rect{x:72,y:114,w:112,h:16},self.small_font,muted,DT_LEFT);
        self.fitted_text(dc,"Network settings",layout::Rect{x:72,y:131,w:112,h:12},self.caption_font,muted,DT_LEFT);
        self.icon(dc,216,103,30,icons::Glyph::Bluetooth,if self.system.devices.connected.is_empty(){muted}else{rgb(95,155,255)},card,None);
        self.fitted_text(dc,"Bluetooth",layout::Rect{x:260,y:94,w:112,h:18},self.small_font,white,DT_LEFT);
        let name=self.system.devices.connected.first().map(String::as_str).unwrap_or(&self.system.devices.bluetooth);
        self.fitted_text(dc,name,layout::Rect{x:260,y:114,w:112,h:16},self.small_font,muted,DT_LEFT);
        self.fitted_text(dc,if self.system.devices.connected.is_empty() {"Device settings"}else{&self.system.devices.bluetooth},layout::Rect{x:260,y:131,w:112,h:12},self.caption_font,muted,DT_LEFT);
        for (y,kind,level,icon) in [(156,osd::Kind::Volume,self.system.audio.volume,if self.system.audio.muted {icons::Glyph::Muted}else{icons::Glyph::Volume}),
            (204,osd::Kind::Brightness,self.system.devices.brightness,icons::Glyph::Brightness)] {
            self.framed_tile(dc,layout::Rect{x:16,y,w:368,h:40},16,card,rgb(35,35,47));self.icon(dc,28,y+9,22,icon,white,card,None);
            let label=if kind==osd::Kind::Volume {"Sound"}else{"Brightness"};
            let label=self.system.error.as_ref().filter(|(error,_)|error.starts_with(if kind==osd::Kind::Volume {"Volume"}else{"Brightness"})).map(|(error,_)|error.as_str()).unwrap_or(label);
            self.fitted_text(dc,label,layout::Rect{x:76,y:y+2,w:240,h:14},self.caption_font,muted,DT_LEFT);
            self.tile(dc,76,y+22,252,6,6,rgb(44,44,57));
            if let Some(level)=level {
                let value=if self.osd.kind==kind && self.osd.visible(std::time::Instant::now()) {self.osd.value(std::time::Instant::now())}else{level as f64};
                let width=(252.*value/100.).round() as i32;if width>0 {self.tile(dc,76,y+22,width,6,6,white);}
                self.smooth_tile(dc,layout::Rect{x:71+width,y:y+20,w:10,h:10},10,white,card);
                self.fitted_text(dc,&format!("{level}%"),layout::Rect{x:334,y:y+5,w:38,h:30},self.small_font,white,DT_RIGHT);
            }else {self.fitted_text(dc,"N/A",layout::Rect{x:334,y:y+5,w:38,h:30},self.small_font,muted,DT_RIGHT);}
        }
    }
    pub(super) unsafe fn effects_view(&self,dc:HDC) {
        let white=rgb(239,239,247);let muted=rgb(143,143,164);
        self.framed_tile(dc,layout::Rect{x:16,y:88,w:112,h:24},10,rgb(18,18,25),rgb(35,35,47));
        self.fitted_text(dc,"Appearance",layout::Rect{x:16,y:88,w:112,h:24},self.small_font,white,DT_CENTER);
        self.fitted_text(dc,"Glass & transparency",layout::Rect{x:144,y:88,w:240,h:24},self.small_font,white,DT_LEFT);
        self.framed_tile(dc,layout::Rect{x:16,y:122,w:368,h:118},18,rgb(18,18,25),rgb(35,35,47));
        self.fitted_text(dc,"Frosted backdrop",layout::Rect{x:28,y:130,w:280,h:20},self.small_font,white,DT_LEFT);
        self.fitted_text(dc,if self.glass.failed.get() {"Glass unavailable on this display"}else if self.transparency==0 {"Opaque at 0% transparency"}else if !self.frosted {"Clear transparency"}else if self.glass.blur_available {"Native blur active"}else{"Clear backdrop / blur may vary by Windows"},layout::Rect{x:28,y:151,w:300,h:16},self.caption_font,muted,DT_LEFT);
        self.tile(dc,336,134,32,18,18,if self.frosted {self.color}else{rgb(50,50,65)});self.tile(dc,if self.frosted {352}else{338},136,14,14,14,white);
        self.fitted_text(dc,"Transparency",layout::Rect{x:28,y:177,w:250,h:16},self.small_font,white,DT_LEFT);
        self.fitted_text(dc,&format!("{}%",self.transparency),layout::Rect{x:302,y:177,w:70,h:16},self.small_font,white,DT_RIGHT);
        self.tile(dc,28,203,344,5,5,rgb(44,44,57));let width=(344*self.transparency/100) as i32;
        if width>0 {self.tile(dc,28,203,width,5,5,self.color);}self.smooth_tile(dc,layout::Rect{x:22+width,y:199,w:13,h:13},13,white,rgb(18,18,25));
        self.fitted_text(dc,"0% opaque   ·   100% clear background",layout::Rect{x:28,y:220,w:344,h:16},self.caption_font,muted,DT_CENTER);
    }
    pub(super) unsafe fn draw_osd(&self,dc:HDC,x:i32,y:i32,w:i32,h:i32,alpha:f64) {
        let blend=|color:u32| {let c=|s:u32|(8.+(((color>>s)&255) as f64-8.)*alpha.clamp(0.,1.)).round() as u32;rgb(c(0),c(8),c(16))};
        let card=blend(rgb(20,20,29));let fg=blend(rgb(239,239,247));
        self.framed_tile(dc,layout::Rect{x,y,w,h},20,card,blend(rgb(48,48,64)));
        let glyph=match self.osd.kind {osd::Kind::Brightness=>icons::Glyph::Brightness,osd::Kind::Volume if self.osd.muted=>icons::Glyph::Muted,_=>icons::Glyph::Volume};
        self.icon(dc,x+14,y+(h-22)/2,22,glyph,fg,card,None);
        let label=if self.osd.kind==osd::Kind::Brightness {"Brightness"}else if self.osd.muted {"Muted"}else{"Volume"};
        self.fitted_text(dc,label,layout::Rect{x:x+48,y:y+6,w:w-108,h:16},self.caption_font,fg,DT_LEFT);
        let value=self.osd.value(std::time::Instant::now());self.fitted_text(dc,&format!("{}%",value.round() as u8),layout::Rect{x:x+w-58,y:y+6,w:44,h:16},self.small_font,fg,DT_RIGHT);
        self.tile(dc,x+48,y+h-16,w-62,5,5,blend(rgb(50,50,66)));let fill=((w-62) as f64*value/100.).round() as i32;
        if fill>0 {self.tile(dc,x+48,y+h-16,fill,5,5,fg);}
    }
}
