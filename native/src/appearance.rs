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
        self.glow=((x-28).clamp(0,168)*100/168) as u32;
    }

    pub unsafe fn appearance_click(&mut self, hwnd: HWND, x:i32, y:i32) {
        if (24..192).contains(&x) && (181..204).contains(&y) {
            let index=((x-24)/24) as usize;
            if let Some(color)=PRESETS.get(index) { self.color=*color; }
        } else if (24..202).contains(&x) && (211..238).contains(&y) {
            self.choose_color(hwnd);
        } else if (20..206).contains(&x) && (253..281).contains(&y) {
            self.dragging_glow=true; SetCapture(hwnd); self.set_glow_from_pointer(x);
        } else if (224..456).contains(&x) && (110..270).contains(&y) {
            match (y-110)/40 {
                0 => self.mascot=!self.mascot,
                1 => self.compact_lyrics=!self.compact_lyrics,
                2 => self.minimal=!self.minimal,
                3 => self.floating=!self.floating,
                _=>{}
            }
        }
        self.save(); self.resize(hwnd); InvalidateRect(hwnd,null(),0);
    }

    pub unsafe fn appearance(&self, dc:HDC) {
        let muted=rgb(133,133,150); let white=rgb(235,235,243);
        self.tile(dc,14,96,200,193,20,rgb(19,19,25));
        self.tile(dc,222,96,234,193,20,rgb(19,19,25));
        self.text(dc,"MASCOT COLOUR",26,106,muted,true);
        self.tile(dc,24,126,180,45,14,rgb(8,8,11));
        self.eyes(dc,80,139);
        for (i,color) in PRESETS.iter().enumerate() {
            let x=24+i as i32*24;
            if *color==self.color { self.tile(dc,x-2,180,24,24,24,white); }
            self.tile(dc,x,182,20,20,20,*color);
        }
        self.tile(dc,24,212,180,25,12,rgb(31,31,40));
        self.tile(dc,33,219,11,11,5,self.color);
        self.text(dc,"Custom colour...",54,216,white,true);
        self.text(dc,&format!("Glow                  {}%",self.glow),26,242,muted,true);
        self.tile(dc,28,265,168,4,4,rgb(52,52,64));
        let width=(168*self.glow/100) as i32;
        if width>0 {self.tile(dc,28,265,width,4,4,self.color);}
        self.tile(dc,23+width,261,12,12,12,self.color);
        self.text(dc,"BEHAVIOUR",234,106,muted,true);
        for (i,(label,hint,on)) in [
            ("Mascot eyes","Clock when disabled",self.mascot),
            ("Compact lyrics","Lyrics while playing",self.compact_lyrics),
            ("Minimal notch","Thin line when idle",self.minimal),
            ("Floating pill","Drag header to move",self.floating),
        ].iter().enumerate() {
            let y=126+i as i32*40;
            self.text(dc,label,234,y,white,true);
            self.text(dc,hint,234,y+15,muted,true);
            self.tile(dc,416,y+5,28,16,16,if *on {self.color} else {rgb(61,61,72)});
            self.tile(dc,if *on {430} else {418},y+7,12,12,12,white);
        }
        self.text(dc,"APPEARANCE  /  Saved on this PC",24,298,muted,true);
    }
}
