use super::*;

const LINES: [&str; 11] = ["we fold the summer light", "into paper birds tonight", "and send them through the sky", "to find you by the sea", "every wing a little song", "that carries us along", "and if the wind forgets our name", "we'll write it in the rain", "paper sky, paper sky", "hold the morning up so high", "don't let the summer say goodbye"];

impl App {
    pub fn is_playing(&self) -> bool { if self.media.available { self.media.playing } else { self.playing } }
    pub fn compact_line(&self) -> String {
        if self.media.available {
            let text = self.lyric_lines.iter().rev().find(|l| l.at <= self.media.position.saturating_mul(1000))
                .map(|l| l.text.as_str()).filter(|s| !s.is_empty()).unwrap_or(&self.media.title);
            shorten(text,34)
        } else { self.lyric().to_string() }
    }
    pub fn lyric(&self) -> &'static str { LINES[(self.position as usize / 18).min(LINES.len()-1)] }

    pub(super) unsafe fn tile(&self, dc: HDC, x: i32, y: i32, w: i32, h: i32, radius: i32, color: u32) {
        let brush = CreateSolidBrush(color);
        let old = SelectObject(dc, brush);
        let pen = SelectObject(dc, GetStockObject(NULL_PEN));
        RoundRect(dc, self.px(x), self.px(y), self.px(x+w), self.px(y+h), self.px(radius), self.px(radius));
        SelectObject(dc, pen);
        SelectObject(dc, old);
        DeleteObject(brush);
    }

    pub unsafe fn cover(&self, dc: HDC, x: i32, y: i32, size: i32) {
        self.cover_on(dc,x,y,size,rgb(8,8,11));
    }
    unsafe fn cover_on(&self,dc:HDC,x:i32,y:i32,size:i32,background:u32) {
        if let Some(image)=self.media.artwork.as_ref().filter(|i|i.width>0 && i.height>0 && i.width<=256 && i.height<=256 && i.bgra.len()==i.width as usize*i.height as usize*4) {
            self.cover_cache.borrow_mut().draw(dc,image,self.px(x),self.px(y),self.px(size) as usize,background,self.px(8) as f64);
            return;
        }
        let mut origin=POINT{x:0,y:0};GetViewportOrgEx(dc,&mut origin);
        let region = CreateRoundRectRgn(self.px(x)+origin.x, self.px(y)+origin.y, self.px(x+size)+origin.x, self.px(y+size)+origin.y, self.px(16), self.px(16));
        SaveDC(dc);
        ExtSelectClipRgn(dc,region,RGN_AND);
        if self.media.available {
            self.tile(dc,x,y,size,size,16,rgb(24,24,32));
            let pen=CreatePen(PS_SOLID,self.px(1).max(1),rgb(132,132,151));let old=SelectObject(dc,pen);
            let brush=SelectObject(dc,GetStockObject(NULL_BRUSH));
            let (cx,cy)=(x+size/2,y+size/2-if size>=64 {8}else{0});
            MoveToEx(dc,self.px(cx+4),self.px(cy-8),std::ptr::null_mut());LineTo(dc,self.px(cx+4),self.px(cy+4));
            LineTo(dc,self.px(cx-3),self.px(cy+4));
            Ellipse(dc,self.px(cx-7),self.px(cy+1),self.px(cx+4),self.px(cy+8));
            SelectObject(dc,brush);SelectObject(dc,old);DeleteObject(pen);
            if size>=64 {self.text(dc,if self.media.artwork_pending {"Loading..."}else{"No artwork"},x+(size-60)/2,y+size/2+12,rgb(132,132,151),true);}
        } else {
        for row in 0..size {
            let t = row as f32 / size as f32;
            let color = rgb((255. - 121.*t) as u32, (184. - 129.*t) as u32, (130. + 75.*t) as u32);
            self.tile(dc, x, y+row, size, 2, 0, color);
        }
        let pen = CreatePen(PS_SOLID, 1, rgb(204,135,200));
        let old_pen = SelectObject(dc, pen);
        let old_brush = SelectObject(dc, GetStockObject(NULL_BRUSH));
        for r in (8..size*2).step_by(10) {
            Ellipse(dc,self.px(x+size-r),self.px(y+size-r),self.px(x+size+r),self.px(y+size+r));
        }
        SelectObject(dc,old_pen); SelectObject(dc,old_brush); DeleteObject(pen);
        }
        RestoreDC(dc,-1); DeleteObject(region);
    }

    pub unsafe fn panel(&self, dc: HDC) {
        let muted=rgb(135,135,151);
        self.framed_tile(dc,layout::Rect{x:16,y:10,w:layout::NAV_RIGHT-16,h:28},14,rgb(17,17,24),rgb(32,32,44));
        if layout::tab_slot(self.tab).is_some() {
            let x=self.tab_pill.sample(std::time::Instant::now()).0.round() as i32;
            self.tile(dc,x,12,layout::TAB_PILL_W,24,12,rgb(39,30,43));
        }
        for (i,(id,label)) in [(0,"Music"),(2,"Focus"),(3,"Water"),(4,"Shelf"),(5,"To-do")].iter().enumerate() {
            let x=layout::TAB_START+i as i32*layout::TAB_STEP;
            let active=self.tab==*id || (*id==0 && self.tab==1);
            let color=if active {rgb(240,237,246)}else if self.hover==layout::Hover::Tab(i) {rgb(215,215,228)}else{muted};
            self.fitted_text(dc,label,layout::Rect{x,y:12,w:layout::TAB_PILL_W,h:24},self.small_font,color,DT_CENTER);
        }
        let bt_bg=if self.tab==9 {rgb(31,35,53)}else if self.hover==layout::Hover::Bluetooth {rgb(29,29,41)}else{rgb(17,17,24)};
        self.framed_tile(dc,layout::Rect{x:layout::BLUETOOTH_LEFT,y:10,w:32,h:28},12,bt_bg,rgb(32,32,44));
        self.icon(dc,layout::BLUETOOTH_LEFT+7,15,18,icons::Glyph::Bluetooth,if self.tab==9 {rgb(126,178,255)}else{rgb(170,170,188)},bt_bg,None);
        let center_bg=if self.tab==7 {rgb(31,35,53)}else if self.hover==layout::Hover::Control {rgb(29,29,41)}else{rgb(17,17,24)};
        self.framed_tile(dc,layout::Rect{x:layout::CONTROL_LEFT,y:10,w:32,h:28},12,center_bg,rgb(32,32,44));
        self.icon(dc,layout::CONTROL_LEFT+7,15,18,icons::Glyph::Control,if self.tab==7 {rgb(126,178,255)}else{rgb(170,170,188)},center_bg,None);
        let bg=if matches!(self.tab,6|8|10|11|12) {rgb(43,29,44)}else if self.hover==layout::Hover::Settings {rgb(29,29,41)}else{rgb(17,17,24)};
        self.framed_tile(dc,layout::Rect{x:layout::SETTINGS_LEFT,y:10,w:32,h:28},12,bg,rgb(32,32,44));
        self.icon(dc,layout::SETTINGS_LEFT+7,15,18,icons::Glyph::Settings,if matches!(self.tab,6|8|10|11|12) {self.color}else{rgb(170,170,188)},bg,None);
        SaveDC(dc);SetViewportOrgEx(dc,0,-self.px(layout::BODY_OFFSET),std::ptr::null_mut());
        self.panel_body(dc);RestoreDC(dc,-1);self.keyboard_focus(dc);
    }

    pub(super) unsafe fn fitted_text(&self,dc:HDC,text:&str,rect:layout::Rect,font:HFONT,color:u32,align:u32) {
        SelectObject(dc,font);SetTextColor(dc,color);
        let mut rect=RECT{left:self.px(rect.x),top:self.px(rect.y),right:self.px(rect.x+rect.w),bottom:self.px(rect.y+rect.h)};
        let text=wide(text);
        DrawTextW(dc,text.as_ptr(),text.len() as i32-1,&mut rect,DT_SINGLELINE|DT_VCENTER|DT_END_ELLIPSIS|DT_NOPREFIX|align);
    }
    pub(super) unsafe fn framed_tile(&self,dc:HDC,rect:layout::Rect,radius:i32,fill:u32,border:u32) {
        self.shapes.borrow_mut().draw(dc,self.px(rect.x),self.px(rect.y),self.px(rect.w) as usize,self.px(rect.h) as usize,self.px(radius),fill,border,rgb(8,8,11));
    }
    pub(super) unsafe fn smooth_tile(&self,dc:HDC,rect:layout::Rect,radius:i32,fill:u32,background:u32) {
        self.shapes.borrow_mut().draw(dc,self.px(rect.x),self.px(rect.y),self.px(rect.w) as usize,self.px(rect.h) as usize,self.px(radius),fill,fill,background);
    }
    pub(super) unsafe fn icon(&self,dc:HDC,x:i32,y:i32,size:i32,glyph:icons::Glyph,color:u32,bg:u32,disc:Option<u32>) {
        self.icons.borrow_mut().draw(dc,self.scale,x,y,size,glyph,color,bg,disc);
    }

    pub(super) unsafe fn refresh_spectrum(&mut self,hwnd:HWND) {
        let enabled=self.spectrum_preferences.enabled && !self.hidden_fullscreen && self.media.available && self.is_playing() && ((!self.expanded && self.compact_lyrics) || (self.expanded && matches!(self.tab,0|1)));
        if self.spectrum.enable(enabled) && enabled {SetTimer(hwnd,9,33,None);}
        self.update_spectrum(hwnd);
    }
    pub(super) unsafe fn update_spectrum(&mut self,hwnd:HWND) {
        let now=std::time::Instant::now();self.spectrum_visual.target(self.spectrum.reading().sensitivity(self.spectrum_preferences.sensitivity),self.spectrum.enabled(),now);
        if !self.motion_enabled {
            let before=self.spectrum_visual.bars;self.spectrum_visual.tick(now,false);
            if before!=self.spectrum_visual.bars {InvalidateRect(hwnd,std::ptr::null(),0);}
        } else if self.spectrum_visual.moving() {self.clock.start(hwnd);}
        if !self.spectrum.enabled() && self.spectrum_visual.flat() {KillTimer(hwnd,9);}
    }
    pub(super) unsafe fn draw_spectrum(&self,dc:HDC,rect:layout::Rect,background:u32) {
        let step=rect.w as f32/spectrum::BANDS as f32;
        for (i,level) in self.spectrum_visual.bars.iter().enumerate() {
            let height=(2.+(rect.h-2) as f32*level).round() as i32;
            let x=rect.x+(i as f32*step).round() as i32;
            let w=((i+1) as f32*step).round() as i32-(i as f32*step).round() as i32-1;
            let color=if *level>0.01 {if self.spectrum_preferences.white {rgb(239,239,247)}else{self.color}}else{rgb(62,62,78)};
            self.smooth_tile(dc,layout::Rect{x,y:rect.y+(rect.h-height)/2,w:w.max(1),h:height},w.max(1),color,background);
        }
    }
    unsafe fn music(&self,dc:HDC) {
        let title=if self.media.available {self.media.title.as_str()}else{"Paper Sky"};
        let artist=if self.media.available {self.media.artist.as_str()}else{"Lumen Coast"};
        let album=if self.media.available {self.media.album.as_str()}else{"Summer Flows"};
        let duration=if self.media.available {self.media.duration.max(1)}else{193};
        let position=if self.media.available {self.media.position}else{self.position}.min(duration);
        let white=rgb(239,239,247);let muted=rgb(143,143,164);let bg=rgb(8,8,11);
        self.framed_tile(dc,layout::MUSIC_CARD,20,rgb(18,18,25),rgb(35,35,47));
        if self.tab==0 {
            self.tile(dc,23,97,74,74,18,rgb(46,46,59));self.cover_on(dc,24,98,72,rgb(18,18,25));
            let show_album=!album.trim().is_empty() && !album.trim().eq_ignore_ascii_case(title.trim());
            let shift=if show_album {0}else{9};
            self.fitted_text(dc,title,layout::Rect{x:112,y:102+shift,w:264,h:28},self.title_font,white,DT_LEFT);
            self.fitted_text(dc,artist,layout::Rect{x:112,y:132+shift,w:264,h:18},self.small_font,rgb(173,173,193),DT_LEFT);
            if show_album {self.fitted_text(dc,album,layout::Rect{x:112,y:152,w:264,h:14},self.caption_font,muted,DT_LEFT);}
        } else {
            self.fitted_text(dc,title,layout::Rect{x:32,y:96,w:336,h:14},self.caption_font,muted,DT_CENTER);
            let (previous,current,next)=if self.media.available {
                let active=self.lyric_lines.iter().rposition(|l|l.at<=self.media.position.saturating_mul(1000)).unwrap_or(0);
                (active.checked_sub(1).and_then(|i|self.lyric_lines.get(i)).map(|l|l.text.as_str()),
                    self.lyric_lines.get(active).map(|l|l.text.as_str()),self.lyric_lines.get(active+1).map(|l|l.text.as_str()))
            } else {
                let active=(self.position as usize/18).min(LINES.len()-1);
                (active.checked_sub(1).and_then(|i|LINES.get(i)).copied(),LINES.get(active).copied(),LINES.get(active+1).copied())
            };
            if let Some(current)=current {
                self.fitted_text(dc,previous.unwrap_or(""),layout::Rect{x:32,y:112,w:336,h:14},self.small_font,rgb(103,103,126),DT_CENTER);
                self.fitted_text(dc,current,layout::Rect{x:32,y:128,w:336,h:22},self.font,self.color,DT_CENTER);
                self.fitted_text(dc,next.unwrap_or(""),layout::Rect{x:32,y:152,w:336,h:14},self.small_font,rgb(103,103,126),DT_CENTER);
            } else {
                self.fitted_text(dc,"No synced lyrics",layout::Rect{x:32,y:120,w:336,h:24},self.font,white,DT_CENTER);
                self.fitted_text(dc,"Lyrics aren't available for this track yet.",layout::Rect{x:32,y:151,w:336,h:16},self.small_font,muted,DT_CENTER);
            }
        }
        self.tile(dc,24,190,352,3,3,rgb(39,39,51));
        let progress=(352*position/duration) as i32;
        if progress>0 {self.tile(dc,24,190,progress,3,3,self.color);}
        if self.hover==layout::Hover::Action(layout::Action::Seek) {
            self.smooth_tile(dc,layout::Rect{x:20+progress,y:188,w:8,h:8},8,theme::TEXT,bg);
        }
        self.fitted_text(dc,&format!("{}:{:02}",position/60,position%60),layout::Rect{x:24,y:197,w:64,h:16},self.caption_font,muted,DT_LEFT);
        let left=duration.saturating_sub(position);
        self.fitted_text(dc,&format!("-{}:{:02}",left/60,left%60),layout::Rect{x:312,y:197,w:64,h:16},self.caption_font,muted,DT_RIGHT);

        self.tile(dc,24,220,48,28,10,rgb(14,14,20));
        let status=if !self.media.available {"DEMO"}else if self.is_playing() {"PLAYING"}else{"PAUSED"};
        if self.spectrum_preferences.enabled && self.media.available && (self.is_playing() || !self.spectrum_visual.flat()) {
            self.draw_spectrum(dc,layout::Rect{x:28,y:224,w:40,h:20},rgb(14,14,20));
        }else {self.fitted_text(dc,status,layout::Rect{x:26,y:220,w:44,h:28},self.caption_font,rgb(119,119,142),DT_CENTER);}
        for (rect,action,glyph) in [(layout::MUSIC_PREVIOUS,layout::MusicAction::Previous,icons::Glyph::Previous),
            (layout::MUSIC_NEXT,layout::MusicAction::Next,icons::Glyph::Next)] {
            let hovered=self.hover==layout::Hover::Music(action);let surface=if hovered {rgb(24,24,34)}else{bg};
            if hovered {self.tile(dc,rect.x,rect.y,rect.w,rect.h,12,surface);}
            self.icon(dc,rect.x+6,rect.y+6,20,glyph,if hovered {white}else{rgb(170,170,190)},surface,None);
        }
        let rect=layout::MUSIC_PLAY;
        let disc=if self.hover==layout::Hover::Music(layout::MusicAction::Play) {rgb(255,255,255)}else{white};
        self.icon(dc,rect.x,rect.y,36,if self.is_playing() {icons::Glyph::Pause}else{icons::Glyph::Play},rgb(16,16,23),bg,Some(disc));
        let rect=layout::MUSIC_LYRICS;
        let surface=if self.tab==1 {rgb(43,27,43)}else if self.hover==layout::Hover::Music(layout::MusicAction::Lyrics) {rgb(30,30,43)}else{rgb(20,20,29)};
        self.framed_tile(dc,rect,12,surface,if self.tab==1 {self.color}else{rgb(34,34,47)});
        self.icon(dc,rect.x+14,rect.y+4,20,icons::Glyph::Lyrics,if self.tab==1 {self.color}else{rgb(165,165,185)},surface,None);
    }

    unsafe fn panel_body(&self,dc:HDC) {
        let muted = rgb(135,135,151);
        let white = rgb(240,240,245);
        match self.tab {
            12=>{self.power_view(dc);return;}
            10=>{self.general_view(dc);return;}
            11=>{self.spectrum_view(dc);return;}
            7|9=>{self.control_view(dc);return;}
            8=>{self.effects_view(dc);return;}
            5 => {self.todo_view(dc); return;}
            4 => {self.shelf_view(dc); return;}
            6 => { self.appearance(dc); return; }
            0|1 => {self.music(dc);}
            2 => {
                self.framed_tile(dc,layout::Rect{x:16,y:88,w:368,h:98},20,theme::CARD,theme::BORDER);
                self.fitted_text(dc,"FOCUS SESSION",layout::Rect{x:28,y:100,w:220,h:14},self.caption_font,theme::MUTED,DT_LEFT);
                let status=if self.focus_end.is_some() {"RUNNING"}else if self.focus_left==0 {"COMPLETE"}else if self.focus_left<1500 {"PAUSED"}else{"READY"};
                self.smooth_tile(dc,layout::Rect{x:276,y:100,w:96,h:18},8,theme::LIFT,theme::CARD);
                self.fitted_text(dc,status,layout::Rect{x:276,y:100,w:96,h:18},self.caption_font,theme::TEXT,DT_CENTER);
                self.fitted_text(dc,&format!("{:02}:{:02}",self.focus_left/60,self.focus_left%60),layout::Rect{x:28,y:118,w:232,h:48},self.metric_font,theme::TEXT,DT_LEFT);
                self.fitted_text(dc,"25-minute session",layout::Rect{x:28,y:166,w:232,h:12},self.caption_font,theme::MUTED,DT_LEFT);
                self.icon(dc,310,132,36,icons::Glyph::Clock,self.color,theme::CARD,None);
                self.action_button(dc,layout::FOCUS_TOGGLE,if self.focus_end.is_some() {"Pause"}else{"Start"},ui::Button::Primary,layout::Action::FocusToggle);
                self.action_button(dc,layout::FOCUS_RESET,"Reset",ui::Button::Secondary,layout::Action::FocusReset);
                self.fitted_text(dc,&format!("{}% done",(1500-self.focus_left.min(1500))*100/1500),layout::Rect{x:272,y:196,w:100,h:32},self.caption_font,theme::MUTED,DT_RIGHT);
                self.fitted_text(dc,"Keeps running when the notch is closed",layout::Rect{x:20,y:240,w:360,h:14},self.caption_font,theme::MUTED,DT_CENTER);
            }
            3 => {
                self.framed_tile(dc,layout::Rect{x:16,y:88,w:368,h:98},20,theme::CARD,theme::BORDER);
                self.fitted_text(dc,"DAILY HYDRATION",layout::Rect{x:28,y:100,w:220,h:14},self.caption_font,theme::MUTED,DT_LEFT);
                self.fitted_text(dc,"GOAL 1.5 L",layout::Rect{x:264,y:100,w:108,h:14},self.caption_font,theme::MUTED,DT_RIGHT);
                self.fitted_text(dc,&format!("{:.1} L",self.water_ml as f32/1000.),layout::Rect{x:28,y:118,w:232,h:48},self.metric_font,theme::TEXT,DT_LEFT);
                self.icon(dc,292,130,22,icons::Glyph::Drop,self.color,theme::CARD,None);
                self.fitted_text(dc,&format!("{}%",self.water_ml.min(1500)*100/1500),layout::Rect{x:316,y:128,w:56,h:28},self.font,theme::TEXT,DT_RIGHT);
                self.tile(dc,28,172,344,5,5,theme::RAIL);let width=(344*self.water_ml.min(1500)/1500) as i32;
                if width>0 {self.tile(dc,28,172,width,5,5,self.color);}
                self.action_button(dc,layout::WATER_BUTTONS[0],"+200 ml",ui::Button::Primary,layout::Action::Water200);
                self.action_button(dc,layout::WATER_BUTTONS[1],"+400 ml",ui::Button::Secondary,layout::Action::Water400);
                self.action_button(dc,layout::WATER_BUTTONS[2],"Reset",ui::Button::Quiet,layout::Action::WaterReset);
                self.fitted_text(dc,"Saved locally · resets each new day",layout::Rect{x:20,y:240,w:360,h:14},self.caption_font,theme::MUTED,DT_CENTER);
            }
            _ => {
                let (title,detail)=match self.tab {
                    2 => ("25:00", "Focus timer"), 3 => ("1.5 / 1.5 L", "Water reminder"),
                    4 => ("Drop files here", "File shelf"), _ => ("Your tasks, nearby", "To-do")
                };
                self.tile(dc,20,90,360,140,20,rgb(19,19,25));
                self.text(dc,detail,32,108,muted,true);
                self.text(dc,title,32,140,white,false);
                self.text(dc,"Native feature not connected yet",32,182,muted,true);
            }
        }
    }
}

pub(super) fn shorten(text: &str, max: usize) -> String {
    if text.chars().count() <= max { text.to_string() }
    else { text.chars().take(max.saturating_sub(1)).chain(std::iter::once('…')).collect() }
}
