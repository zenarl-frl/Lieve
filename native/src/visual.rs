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
        let region = CreateRoundRectRgn(self.px(x), self.px(y), self.px(x+size), self.px(y+size), self.px(16), self.px(16));
        SaveDC(dc);
        SelectClipRgn(dc, region);
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
        RestoreDC(dc,-1); DeleteObject(region);
    }

    pub unsafe fn panel(&self, dc: HDC) {
        let title = if self.media.available { self.media.title.as_str() } else { "Paper Sky" };
        let artist = if self.media.available { self.media.artist.as_str() } else { "Lumen Coast" };
        let album = if self.media.available { self.media.album.as_str() } else { "Summer Flows" };
        let position = if self.media.available { self.media.position } else { self.position };
        let duration = if self.media.available { self.media.duration.max(1) } else { 193 };
        let muted = rgb(135,135,151);
        let white = rgb(240,240,245);
        self.cover(dc,14,14,26);
        // Clip metadata to the left lane; its length must never move/overlap the eyes.
        SaveDC(dc);
        IntersectClipRect(dc,self.px(50),0,self.px(188),self.px(48));
        self.text(dc,&shorten(if self.tab==6 {"Appearance"} else {title},20),50,12,white,true);
        self.text(dc,&shorten(if self.tab==6 {"Make Lieve yours"} else {artist},20),50,29,muted,true);
        RestoreDC(dc,-1);
        self.eyes(dc,(470-68)/2,18);
        let mut time=std::mem::zeroed::<SYSTEMTIME>();
        GetLocalTime(&mut time);
        self.text(dc,&format!("{:02}:{:02}",time.wHour,time.wMinute),322,21,muted,true);
        self.tile(dc,385,14,28,28,10,if self.pinned { rgb(57,27,31) } else { rgb(24,24,30) });
        self.text(dc,"P",394,18,if self.pinned { self.color } else { muted },true);
        self.tile(dc,423,14,28,28,10,rgb(24,24,30));
        self.text(dc,"...",430,17,muted,true);
        self.tile(dc,12,50,446,35,20,rgb(25,25,31));
        for (i,label) in ["Music","Lyrics","Focus","Water","Shelf","To-do"].iter().enumerate() {
            let x = 16+i as i32*74;
            if i == self.tab { self.tile(dc,x,54,70,27,15,rgb(57,28,33)); }
            self.text(dc,label,x+13,60,if i==self.tab { self.color } else { muted },true);
        }
        match self.tab {
            5 => {self.todo_view(dc); return;}
            4 => {self.shelf_view(dc); return;}
            6 => { self.appearance(dc); return; }
            0 => {
                self.cover(dc,15,145,104);
                self.text(dc,&shorten(album,36),138,126,muted,true);
                SelectObject(dc,self.title_font); SetTextColor(dc,white);
                let title=wide(&shorten(title,23)); TextOutW(dc,self.px(138),self.px(147),title.as_ptr(),title.len() as i32-1);
                self.text(dc,&shorten(artist,38),138,176,muted,true);
                self.tile(dc,138,207,315,3,3,rgb(44,44,51));
                self.tile(dc,138,207,(315*position.min(duration)/duration) as i32,3,3,self.color);
                self.text(dc,&format!("{}:{:02}",position/60,position%60),138,216,muted,true);
                let left=duration.saturating_sub(position);
                self.text(dc,&format!("-{}:{:02}",left/60,left%60),411,216,muted,true);
                self.tile(dc,170,243,34,34,34,self.color);
                self.text(dc,if self.is_playing() { "II" } else { ">" },181,250,rgb(9,9,12),true);
                self.text(dc,"|<",141,250,white,true);
                self.text(dc,">|",217,250,white,true);
                self.text(dc,if self.media.available { "Windows media" } else { "Demo playback" },350,251,muted,true);
            }
            1 => {
                if self.media.available {
                    self.text(dc,"SYNCED LYRICS / LRCLIB",24,104,muted,true);
                    if self.lyric_lines.is_empty() {
                        self.text(dc,"No synced lyrics available",24,158,white,false);
                        self.text(dc,"The compact notch shows the song title instead.",24,194,muted,true);
                    } else {
                        let active=self.lyric_lines.iter().rposition(|l| l.at<=self.media.position.saturating_mul(1000)).unwrap_or(0);
                        let start=active.saturating_sub(2);
                        for (row,i) in (start..(start+5).min(self.lyric_lines.len())).enumerate() {
                            self.text(dc,&shorten(&self.lyric_lines[i].text,43),28,134+row as i32*28,if i==active {self.color} else {muted},false);
                        }
                    }
                    return;
                }
                self.text(dc,"SYNCED LYRICS / DEMO",24,104,muted,true);
                let active=(self.position as usize/18).min(LINES.len()-1);
                let start=active.saturating_sub(2);
                for (row,i) in (start..(start+5).min(LINES.len())).enumerate() {
                    self.text(dc,LINES[i],28,134+row as i32*28,if i==active { self.color } else { muted },false);
                }
            }
            2 => {
                self.text(dc,"FOCUS TIME",28,112,muted,true);
                self.text(dc,&format!("{:02}:{:02}",self.focus_left/60,self.focus_left%60),28,157,white,false);
                self.text(dc,"Keeps running when the notch is closed",28,195,muted,true);
                self.tile(dc,24,237,120,34,16,self.color);
                self.text(dc,if self.focus_end.is_some() {"Pause"} else {"Start"},56,245,rgb(8,8,11),true);
                self.tile(dc,158,237,95,34,16,rgb(30,30,38));
                self.text(dc,"Reset",185,245,white,true);
            }
            3 => {
                self.text(dc,"TODAY / WATER INTAKE",28,112,muted,true);
                self.text(dc,&format!("{:.1} / 1.5 L",self.water_ml as f32/1000.),28,153,white,false);
                self.tile(dc,28,199,410,8,8,rgb(36,36,44));
                let width=(410*self.water_ml.min(1500)/1500) as i32;
                if width>0 { self.tile(dc,28,199,width,8,8,self.color); }
                for (x,label) in [(24,"+200 ml"),(158,"+400 ml"),(292,"Reset")] {
                    self.tile(dc,x,237,120,34,16,rgb(36,26,32));
                    self.text(dc,label,x+28,245,self.color,true);
                }
                self.text(dc,"Saved locally · resets each new day",28,278,muted,true);
            }
            _ => {
                let (title,detail)=match self.tab {
                    2 => ("25:00", "Focus timer"), 3 => ("1.5 / 1.5 L", "Water reminder"),
                    4 => ("Drop files here", "File shelf"), _ => ("Your tasks, nearby", "To-do")
                };
                self.tile(dc,24,111,422,129,22,rgb(19,19,25));
                self.text(dc,detail,43,130,muted,true);
                self.text(dc,title,43,163,white,false);
                self.text(dc,"Visual preview — native feature not connected yet",43,204,muted,true);
            }
        }
        self.text(dc,"LIEVE  /  NATIVE DESIGN PREVIEW",24,298,rgb(89,89,103),true);
    }
}

pub(super) fn shorten(text: &str, max: usize) -> String {
    if text.chars().count() <= max { text.to_string() }
    else { text.chars().take(max.saturating_sub(1)).chain(std::iter::once('…')).collect() }
}
