use super::*;

const PRESETS: [u32;7] = [0x3636E2,0x1F12C1,0x2B5CFF,0xFF4D9B,0x5EC522,0xFFE77F,0xF2F2F2];

impl App {
    pub unsafe fn choose_color(&mut self, hwnd: HWND) {
        self.menu_open=true;
        let mut colors=[self.color;16];
        let mut picker=std::mem::zeroed::<CHOOSECOLORW>();
        picker.lStructSize=std::mem::size_of::<CHOOSECOLORW>() as u32;
        picker.hwndOwner=hwnd; picker.rgbResult=self.color;
        picker.lpCustColors=colors.as_mut_ptr(); picker.Flags=CC_FULLOPEN|CC_RGBINIT;
        if ChooseColorW(&mut picker)!=0 { self.color=picker.rgbResult; }
        self.menu_open=false;
    }

    pub fn set_glow_from_pointer(&mut self, x: i32) {
        self.glow=((x-24).clamp(0,144)*100/144) as u32;
    }

    pub unsafe fn appearance_click(&mut self, hwnd: HWND, x:i32, y:i32) {
        if (304..376).contains(&x) && (90..110).contains(&y) {self.tab=8;self.resize(hwnd);return;}
        if (204..292).contains(&x) && (90..110).contains(&y) {self.tab=10;self.autostart_enabled=autostart::enabled().unwrap_or(false);self.resize(hwnd);return;}
        if (24..164).contains(&x) && (164..186).contains(&y) {
            let index=((x-24)/20) as usize;
            if let Some(color)=PRESETS.get(index) { self.color=*color; }
        } else if (24..176).contains(&x) && (192..216).contains(&y) {
            self.choose_color(hwnd);
        } else if (20..174).contains(&x) && (229..248).contains(&y) {
            self.dragging_glow=true; SetCapture(hwnd); self.set_glow_from_pointer(x);
        } else if (196..384).contains(&x) && (112..242).contains(&y) {
            match (y-112)/26 {
                0 => self.mascot=!self.mascot,
                1 => self.compact_lyrics=!self.compact_lyrics,
                2 => self.minimal=!self.minimal,
                3 => self.floating=!self.floating,
                4 => {self.set_grin(hwnd,!self.devil_grin);return;}
                _=>{}
            }
        }else{return;}
        self.save(); self.resize(hwnd); InvalidateRect(hwnd,null(),0);
    }

    pub unsafe fn appearance(&self, dc:HDC) {
        let muted=theme::MUTED;let white=theme::TEXT;
        self.framed_tile(dc,layout::Rect{x:16,y:88,w:168,h:160},18,theme::CARD,theme::BORDER);
        self.framed_tile(dc,layout::Rect{x:196,y:88,w:188,h:160},18,theme::CARD,theme::BORDER);
        self.fitted_text(dc,"MASCOT COLOUR",layout::Rect{x:24,y:98,w:152,h:14},self.caption_font,muted,DT_LEFT);
        self.tile(dc,24,118,152,38,12,rgb(8,8,11));
        SaveDC(dc); IntersectClipRect(dc,self.px(24),self.px(118),self.px(176),self.px(156));
        self.eyes(dc,66,127); RestoreDC(dc,-1);
        for (i,color) in PRESETS.iter().enumerate() {
            let x=24+i as i32*20;
            if *color==self.color { self.tile(dc,x-1,165,20,20,20,white); }
            self.tile(dc,x,166,18,18,18,*color);
        }
        self.tile(dc,24,192,152,24,12,rgb(31,31,40));
        let label=wide("Custom colour");let mut extent=std::mem::zeroed::<SIZE>();SelectObject(dc,self.small_font);
        GetTextExtentPoint32W(dc,label.as_ptr(),label.len() as i32-1,&mut extent);
        let text_width=(extent.cx as f32/self.scale).ceil() as i32;let left=24+(152-text_width-18)/2;
        self.tile(dc,left,199,10,10,5,self.color);
        self.fitted_text(dc,"Custom colour",layout::Rect{x:left+18,y:192,w:text_width,h:24},self.small_font,white,DT_LEFT);
        self.fitted_text(dc,"Glow",layout::Rect{x:24,y:218,w:70,h:16},self.small_font,muted,DT_LEFT);
        self.fitted_text(dc,&format!("{}%",self.glow),layout::Rect{x:94,y:218,w:74,h:16},self.small_font,muted,DT_RIGHT);
        self.tile(dc,24,238,144,3,3,rgb(52,52,64));
        let width=(144*self.glow/100) as i32;
        if width>0 {self.tile(dc,24,238,width,3,3,self.color);}
        self.tile(dc,19+width,234,11,11,11,self.color);
        self.smooth_tile(dc,layout::Rect{x:204,y:92,w:88,h:18},9,rgb(31,31,42),theme::CARD);
        self.fitted_text(dc,"System",layout::Rect{x:204,y:92,w:88,h:18},self.caption_font,white,DT_CENTER);
        self.smooth_tile(dc,layout::Rect{x:304,y:92,w:72,h:18},9,rgb(31,31,42),theme::CARD);
        self.fitted_text(dc,"Glass",layout::Rect{x:304,y:92,w:72,h:18},self.caption_font,white,DT_CENTER);
        for (i,(label,hint,on)) in [
            ("Mascot eyes","Clock if off",self.mascot),
            ("Compact lyrics","Lyrics in mini notch",self.compact_lyrics),
            ("Minimal notch","Thin line when idle",self.minimal),
            ("Floating pill","Drag header to move",self.floating),
            ("Devil grin","Sharp teeth smile",self.devil_grin),
        ].iter().enumerate() {
            let y=112+i as i32*26;
            self.fitted_text(dc,label,layout::Rect{x:208,y:y+1,w:128,h:14},self.small_font,white,DT_LEFT);
            self.fitted_text(dc,hint,layout::Rect{x:208,y:y+15,w:128,h:11},self.caption_font,muted,DT_LEFT);
            self.toggle_switch(dc,layout::Rect{x:348,y:y+5,w:28,h:16},*on,theme::CARD);
        }
    }
}
