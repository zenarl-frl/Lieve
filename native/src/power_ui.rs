use super::*;
impl App {
    pub(super) unsafe fn refresh_power(&mut self,hwnd:HWND) {
        let snapshot=power::read();let changed=snapshot!=self.power.snapshot;
        let low=self.power.update(snapshot);let event=self.power.take_change();
        if let Some(event)=event {self.show_power(hwnd,if event==power::Event::Connected {osd::Kind::PowerConnected}else{osd::Kind::PowerDisconnected},snapshot,false);}
        if let Some(percent)=low {
            self.show_power(hwnd,osd::Kind::LowBattery,snapshot,false);
            let mut notice=std::mem::zeroed::<NOTIFYICONDATAW>();notice.cbSize=std::mem::size_of::<NOTIFYICONDATAW>() as u32;
            notice.hWnd=hwnd;notice.uID=1;notice.uFlags=NIF_INFO;notice.dwInfoFlags=NIIF_WARNING;
            let title=wide("Lieve — low battery");notice.szInfoTitle[..title.len()].copy_from_slice(&title);
            let text=wide(&format!("Battery at {percent}%. Connect your charger when you can."));notice.szInfo[..text.len()].copy_from_slice(&text);Shell_NotifyIconW(NIM_MODIFY,&notice);
        }
        if changed && self.expanded {InvalidateRect(hwnd,null(),0);}
    }
    pub(super) unsafe fn show_power(&mut self,hwnd:HWND,kind:osd::Kind,snapshot:power::Snapshot,preview:bool) {
        if self.hidden_fullscreen {return;}
        self.osd.show_power(kind,snapshot.percent,if preview {kind==osd::Kind::PowerConnected}else{snapshot.charging},preview,std::time::Instant::now(),self.motion_enabled);
        SetTimer(hwnd,8,3000,None);self.resize(hwnd);self.clock.start(hwnd);
    }
    pub(super) fn osd_overlay(&self)->bool {
        self.osd.opacity(std::time::Instant::now())>0. && (self.osd.kind.is_power() || !matches!(self.tab,7|8|9|10|11|12))
    }
    pub(super) unsafe fn battery_icon(&self,dc:HDC,rect:layout::Rect,level:Option<f64>,charging:bool,color:u32,bg:u32,bolt:u32) {
        let body=layout::Rect{w:rect.w-3,..rect};self.smooth_tile(dc,body,4,color,bg);
        self.smooth_tile(dc,layout::Rect{x:body.x+1,y:body.y+1,w:body.w-2,h:body.h-2},2,bg,color);
        self.smooth_tile(dc,layout::Rect{x:body.x+body.w,y:body.y+body.h/3,w:3,h:body.h/3},2,color,bg);
        if let Some(level)=level {
            let width=((body.w-4) as f64*level.clamp(0.,100.)/100.).round() as i32;
            if width>0 {self.smooth_tile(dc,layout::Rect{x:body.x+2,y:body.y+2,w:width,h:body.h-4},2,color,bg);}
        }
        if charging {
            let cx=rect.x+body.w/2;let point=|x:f64,y:f64|POINT{x:self.px(cx)+(x*rect.h as f64*self.scale as f64).round() as i32,y:self.px(rect.y)+(y*rect.h as f64*self.scale as f64).round() as i32};
            let points=[point(0.05,0.16),point(-0.22,0.55),point(0.,0.55),point(-0.06,0.86),point(0.24,0.42),point(0.02,0.42)];
            let brush=CreateSolidBrush(bolt);let old=SelectObject(dc,brush);let pen=SelectObject(dc,GetStockObject(NULL_PEN));Polygon(dc,points.as_ptr(),6);SelectObject(dc,pen);SelectObject(dc,old);DeleteObject(brush);
        }
    }
    pub(super) unsafe fn power_view(&self,dc:HDC) {
        let card=theme::CARD;let white=theme::TEXT;let muted=theme::MUTED;let snapshot=self.power.snapshot;
        self.framed_tile(dc,layout::Rect{x:16,y:88,w:112,h:24},10,card,theme::BORDER);
        self.fitted_text(dc,"System",layout::Rect{x:16,y:88,w:112,h:24},self.small_font,white,DT_CENTER);
        self.fitted_text(dc,"Power & battery",layout::Rect{x:144,y:88,w:240,h:24},self.font,white,DT_LEFT);
        self.framed_tile(dc,layout::Rect{x:16,y:122,w:368,h:78},18,card,theme::BORDER);
        let status=if snapshot.present==Some(false) {"NO BATTERY"}else if snapshot.charging {"CHARGING"}else if snapshot.source==power::Source::Ac {"POWER CONNECTED"}else if snapshot.source==power::Source::Battery {"ON BATTERY"}else{"POWER UNKNOWN"};
        self.fitted_text(dc,status,layout::Rect{x:28,y:130,w:232,h:16},self.caption_font,muted,DT_LEFT);
        let value=snapshot.percent.map(|p|format!("{p}%")).unwrap_or_else(||if snapshot.present==Some(false) {"No battery".into()}else{"—%".into()});
        self.fitted_text(dc,&value,layout::Rect{x:28,y:148,w:232,h:44},self.metric_font,white,DT_LEFT);
        self.battery_icon(dc,layout::Rect{x:292,y:152,w:68,h:30},snapshot.percent.map(|p|p as f64),snapshot.charging,if snapshot.charging {rgb(65,174,121)}else{white},card,white);
        for (x,label) in [(20,"Preview plug"),(204,"Preview unplug")] {
            self.framed_tile(dc,layout::Rect{x,y:210,w:176,h:30},12,theme::LIFT,theme::BORDER);
            self.fitted_text(dc,label,layout::Rect{x,y:210,w:176,h:30},self.small_font,white,DT_CENTER);
        }
        self.fitted_text(dc,"Preview changes animation only, not system power.",layout::Rect{x:16,y:246,w:368,h:12},self.caption_font,muted,DT_CENTER);
    }
    pub(super) unsafe fn power_click(&mut self,hwnd:HWND,x:i32,y:i32) {
        if (16..128).contains(&x) && (88..112).contains(&y) {self.tab=10;self.resize(hwnd);}
        else if (210..240).contains(&y) && ((20..196).contains(&x)||(204..380).contains(&x)) {
            self.show_power(hwnd,if x<200 {osd::Kind::PowerConnected}else{osd::Kind::PowerDisconnected},self.power.snapshot,true);
        }
    }
}
