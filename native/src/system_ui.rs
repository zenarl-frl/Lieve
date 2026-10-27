use super::*;
#[derive(Clone,Copy,PartialEq,Eq)] pub enum Drag {Volume,Brightness,Transparency,Sensitivity}
pub const VOLUME:layout::Rect=layout::Rect{x:76,y:168,w:252,h:24};
pub const BRIGHTNESS:layout::Rect=layout::Rect{x:76,y:216,w:252,h:24};
pub const TRANSPARENCY:layout::Rect=layout::Rect{x:28,y:194,w:344,h:24};
pub fn percent(x:i32,rect:layout::Rect)->u8 {((x-rect.x).clamp(0,rect.w)*100/rect.w) as u8}
impl App {
    pub(super) unsafe fn open_windows_settings(&mut self,hwnd:HWND,uri:&str) {
        ShellExecuteW(hwnd,wide("open").as_ptr(),wide(uri).as_ptr(),null(),null(),SW_SHOWNORMAL);
    }
    pub(super) unsafe fn toggle_control(&mut self,hwnd:HWND) {
        if self.tab==7 {self.tab=self.control_return;}else{self.control_return=self.tab;self.tab=7;}
        self.system_controller.refresh();self.resize(hwnd);
    }
    pub(super) unsafe fn toggle_bluetooth(&mut self,hwnd:HWND) {
        if self.tab==9 {self.tab=self.bluetooth_return;}else{self.bluetooth_return=self.tab;self.tab=9;self.bluetooth_offset=0;}
        self.system_controller.refresh_bluetooth();self.resize(hwnd);
    }
    pub(super) unsafe fn show_osd(&mut self,hwnd:HWND,kind:osd::Kind,value:u8,muted:bool) {
        if self.hidden_fullscreen {return;}
        let now=std::time::Instant::now();self.osd.show(kind,value,muted,now,self.motion_enabled);
        SetTimer(hwnd,8,2000,None);self.resize(hwnd);self.clock.start(hwnd);
    }
    pub(super) unsafe fn system_slider(&mut self,hwnd:HWND,kind:Drag,x:i32) {
        let rect=match kind {Drag::Volume=>VOLUME,Drag::Brightness=>BRIGHTNESS,Drag::Transparency=>TRANSPARENCY,Drag::Sensitivity=>spectrum_ui::SENSITIVITY};
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
            Drag::Sensitivity=>{self.spectrum_preferences.sensitivity=50+((x-rect.x).clamp(0,rect.w) as u32*150/rect.w as u32);self.update_spectrum(hwnd);},
        }
        InvalidateRect(hwnd,null(),0);
    }
    unsafe fn start_system_drag(&mut self,hwnd:HWND,kind:Drag,x:i32) {
        self.system_drag=Some(kind);SetCapture(hwnd);self.system_slider(hwnd,kind,x);
    }
    pub(super) unsafe fn control_click(&mut self,hwnd:HWND,x:i32,y:i32) {
        if self.tab==9 {
            if (88..112).contains(&y) && (16..128).contains(&x) {self.tab=self.bluetooth_return;self.resize(hwnd);}
            else if (88..112).contains(&y) && (300..384).contains(&x) {self.system_controller.refresh_bluetooth();}
            else if (242..258).contains(&y) && (16..250).contains(&x) {self.open_windows_settings(hwnd,"ms-settings:bluetooth");}
            return;
        }
        if (88..148).contains(&y) && (16..196).contains(&x) {self.open_windows_settings(hwnd,"ms-settings:network-wifi");}
        else if (88..148).contains(&y) && (204..384).contains(&x) {self.toggle_bluetooth(hwnd);}
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
                system_control::Update::Wifi(name,connected)=>{self.system.devices.wifi=name;self.system.devices.wifi_connected=connected;}
                system_control::Update::Bluetooth(snapshot)=>{
                    self.system.devices.bluetooth=snapshot.status;self.system.devices.connected=snapshot.devices;
                    self.bluetooth_offset=self.bluetooth_offset.min(self.system.devices.connected.len().saturating_sub(4));
                },
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
            && !self.osd.visible(std::time::Instant::now()) && !(self.is_playing() && self.compact_lyrics) {self.resize(hwnd);}
        if changed && self.expanded {InvalidateRect(hwnd,null(),0);}
    }
    pub(super) unsafe fn control_view(&self,dc:HDC) {
        let white=rgb(239,239,247);let muted=rgb(143,143,164);let card=rgb(18,18,25);
        if self.tab==9 {
            self.framed_tile(dc,layout::Rect{x:16,y:88,w:112,h:24},10,card,rgb(35,35,47));
            self.fitted_text(dc,"Back",layout::Rect{x:16,y:88,w:112,h:24},self.small_font,white,DT_CENTER);
            self.fitted_text(dc,"Bluetooth",layout::Rect{x:132,y:88,w:156,h:24},self.font,white,DT_LEFT);
            self.framed_tile(dc,layout::Rect{x:300,y:88,w:84,h:24},10,card,theme::BORDER);
            self.fitted_text(dc,"Refresh",layout::Rect{x:300,y:88,w:84,h:24},self.small_font,white,DT_CENTER);
            if self.system.devices.connected.is_empty() {
                self.framed_tile(dc,layout::Rect{x:16,y:120,w:368,h:100},18,card,theme::BORDER);
                self.icon(dc,186,134,28,icons::Glyph::Bluetooth,muted,card,None);
                self.fitted_text(dc,&self.system.devices.bluetooth,layout::Rect{x:28,y:176,w:344,h:24},self.small_font,muted,DT_CENTER);
            }
            for (row,device) in self.system.devices.connected.iter().skip(self.bluetooth_offset).take(4).enumerate() {
                let y=116+row as i32*31;self.framed_tile(dc,layout::Rect{x:16,y,w:368,h:29},10,card,theme::BORDER);
                self.icon(dc,28,y+6,18,icons::Glyph::Bluetooth,rgb(126,178,255),card,None);
                self.fitted_text(dc,&device.name,layout::Rect{x:56,y:y+2,w:220,h:14},self.small_font,white,DT_LEFT);
                self.fitted_text(dc,if device.battery.is_some() {"Connected · Windows battery reading"}else{"Connected · Battery not reported"},layout::Rect{x:56,y:y+17,w:220,h:11},self.caption_font,muted,DT_LEFT);
                let color=if device.battery.is_some_and(|p|p<=20) {rgb(255,180,100)}else{white};
                self.battery_icon(dc,layout::Rect{x:286,y:y+9,w:22,h:12},device.battery.map(|p|p as f64),false,color,card,white);
                let level=device.battery.map(|p|format!("{p}%")).unwrap_or_else(||"—".into());
                self.fitted_text(dc,&level,layout::Rect{x:316,y:y+4,w:56,h:22},self.small_font,color,DT_RIGHT);
            }
            self.fitted_text(dc,"Windows Bluetooth settings",layout::Rect{x:16,y:242,w:234,h:16},self.caption_font,self.color,DT_LEFT);
            let total=self.system.devices.connected.len();let count=if total>0 {format!("{}–{} / {total}",self.bluetooth_offset+1,(self.bluetooth_offset+4).min(total))}else{self.system.devices.bluetooth.clone()};
            self.fitted_text(dc,&count,layout::Rect{x:254,y:242,w:130,h:16},self.caption_font,muted,DT_RIGHT);return;
        }
        for x in [16,204] {self.framed_tile(dc,layout::Rect{x,y:88,w:180,h:60},18,card,rgb(35,35,47));}
        let wifi=if self.system.devices.wifi_connected {rgb(95,155,255)}else{muted};
        self.icon(dc,28,103,30,icons::Glyph::Wifi,wifi,card,None);
        self.fitted_text(dc,"Wi-Fi",layout::Rect{x:72,y:94,w:112,h:18},self.small_font,white,DT_LEFT);
        self.fitted_text(dc,&self.system.devices.wifi,layout::Rect{x:72,y:114,w:112,h:16},self.small_font,muted,DT_LEFT);
        self.fitted_text(dc,"Network settings",layout::Rect{x:72,y:131,w:112,h:12},self.caption_font,muted,DT_LEFT);
        self.icon(dc,216,103,30,icons::Glyph::Bluetooth,if self.system.devices.connected.is_empty(){muted}else{rgb(95,155,255)},card,None);
        self.fitted_text(dc,"Bluetooth",layout::Rect{x:260,y:94,w:112,h:18},self.small_font,white,DT_LEFT);
        let name=self.system.devices.connected.first().map(|d|d.name.as_str()).unwrap_or(&self.system.devices.bluetooth);
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
        self.fitted_text(dc,"Glass & transparency",layout::Rect{x:144,y:88,w:240,h:24},self.font,white,DT_LEFT);
        self.framed_tile(dc,layout::Rect{x:16,y:122,w:368,h:118},18,rgb(18,18,25),rgb(35,35,47));
        self.fitted_text(dc,"Frosted backdrop",layout::Rect{x:28,y:130,w:280,h:20},self.small_font,white,DT_LEFT);
        self.fitted_text(dc,if self.glass.failed.get() {"Glass unavailable on this display"}else if self.transparency==0 {"Opaque at 0% transparency"}else if !self.frosted {"Clear transparency"}else if self.glass.blur_available {"Native blur active"}else{"Clear backdrop / blur may vary by Windows"},layout::Rect{x:28,y:151,w:300,h:16},self.caption_font,muted,DT_LEFT);
        self.toggle_switch(dc,layout::Rect{x:336,y:134,w:32,h:18},self.frosted,theme::CARD);
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
        if self.osd.kind.is_power() {
            let now=std::time::Instant::now();let color=blend(if self.osd.kind==osd::Kind::LowBattery {rgb(255,180,100)}else if self.osd.charging {rgb(65,174,121)}else{theme::TEXT});
            self.battery_icon(dc,layout::Rect{x:x+12,y:y+(h-14)/2,w:26,h:14},self.osd.power_level.map(|_|self.osd.value(now)),self.osd.charging,color,card,fg);
            let label=match self.osd.kind {osd::Kind::PowerConnected if self.osd.charging=>"Charging",osd::Kind::PowerConnected=>"Power connected",osd::Kind::PowerDisconnected=>"On battery",_=>"Low battery"};
            let label=if self.osd.preview {format!("Preview · {label}")}else{label.into()};
            self.fitted_text(dc,&label,layout::Rect{x:x+48,y:y+6,w:w-108,h:16},self.caption_font,fg,DT_LEFT);
            let percent=self.osd.power_level.map(|p|format!("{p}%")).unwrap_or_else(||"—%".into());
            self.fitted_text(dc,&percent,layout::Rect{x:x+w-58,y:y+6,w:44,h:16},self.small_font,fg,DT_RIGHT);
            let bar=w-62;self.tile(dc,x+48,y+h-16,bar,5,5,blend(theme::RAIL));
            if self.osd.power_level.is_some() {
                let fill=(bar as f64*self.osd.value(now)/100.).round() as i32;
                if fill>0 {self.tile(dc,x+48,y+h-16,fill,5,5,color);}
                if let Some(phase)=self.osd.sweep(now) {let at=(phase*fill as f64) as i32;let width=12.min(fill-at);if width>0 {self.tile(dc,x+48+at,y+h-16,width,5,5,fg);}}
            }return;
        }
        let glyph=match self.osd.kind {osd::Kind::Brightness=>icons::Glyph::Brightness,osd::Kind::Volume if self.osd.muted=>icons::Glyph::Muted,_=>icons::Glyph::Volume};
        self.icon(dc,x+14,y+(h-22)/2,22,glyph,fg,card,None);
        let label=if self.osd.kind==osd::Kind::Brightness {"Brightness"}else if self.osd.muted {"Muted"}else{"Volume"};
        self.fitted_text(dc,label,layout::Rect{x:x+48,y:y+6,w:w-108,h:16},self.caption_font,fg,DT_LEFT);
        let value=self.osd.value(std::time::Instant::now());self.fitted_text(dc,&format!("{}%",value.round() as u8),layout::Rect{x:x+w-58,y:y+6,w:44,h:16},self.small_font,fg,DT_RIGHT);
        self.tile(dc,x+48,y+h-16,w-62,5,5,blend(rgb(50,50,66)));let fill=((w-62) as f64*value/100.).round() as i32;
        if fill>0 {self.tile(dc,x+48,y+h-16,fill,5,5,fg);}
    }
}
