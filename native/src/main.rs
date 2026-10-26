#![windows_subsystem = "windows"]

mod settings;
mod visual;
mod media;
mod lyrics;
mod water;
mod animation;
mod appearance;
mod shelf;
mod todo;
mod power;
mod mascot;
mod frame_clock;
mod render;
mod layout;
mod artwork;
mod icons;
mod restart;
mod system_control;
mod system_ui;
mod osd;
mod glass;
use settings::Settings;
use std::{cell::RefCell, ptr::{null, null_mut}};
use std::sync::atomic::{AtomicU32, Ordering};
use windows_sys::Win32::{
    Foundation::*,
    Graphics::Gdi::*,
    System::LibraryLoader::GetModuleHandleW,
    UI::{HiDpi::*, Input::KeyboardAndMouse::*, WindowsAndMessaging::*, Shell::*, Controls::Dialogs::*},
};

fn wide(s: &str) -> Vec<u16> { s.encode_utf16().chain(Some(0)).collect() }
const WM_MOUSELEAVE: u32 = 0x02A3;
const WM_TRAY: u32 = WM_APP + 1;
const WM_FRAME: u32 = WM_APP + 2;
static TASKBAR_CREATED: AtomicU32 = AtomicU32::new(0);
fn rgb(r: u32, g: u32, b: u32) -> u32 { r | g << 8 | b << 16 }

unsafe fn tray(hwnd: HWND, operation: u32) -> bool {
    let mut data = std::mem::zeroed::<NOTIFYICONDATAW>();
    data.cbSize = std::mem::size_of::<NOTIFYICONDATAW>() as u32;
    data.hWnd = hwnd;
    data.uID = 1;
    data.uFlags = NIF_ICON | NIF_MESSAGE | NIF_TIP;
    data.uCallbackMessage = WM_TRAY;
    data.hIcon = LoadIconW(null_mut(), IDI_APPLICATION);
    let tip = wide("Lieve — right-click for settings / exit");
    data.szTip[..tip.len()].copy_from_slice(&tip);
    Shell_NotifyIconW(operation, &data) != 0
}

struct App {
    clock: frame_clock::FrameClock,
    surfaces: RefCell<render::Surfaces>,
    eye_surface: RefCell<render::Surface>,
    eye_raster: RefCell<render::Raster>,
    cover_cache: RefCell<render::CoverCache>,
    shapes: RefCell<render::ShapeCache>,
    glass:glass::Glass,
    glass_raster:RefCell<render::Raster>,
    glass_upload:RefCell<render::Surface>,
    transparency:u32,
    frosted:bool,
    system:system_control::Snapshot,
    system_controller:system_control::Controller,
    system_updates:std::sync::mpsc::Receiver<system_control::Update>,
    system_drag:Option<system_ui::Drag>,
    osd:osd::Hud,
    control_return:usize,
    icons: RefCell<icons::Renderer>,
    hover: layout::Hover,
    region_shape: Option<(i32,i32,i32)>,
    power: power::Monitor,
    tasks: Vec<todo::Task>,
    task_offset: usize,
    shelf: Vec<std::path::PathBuf>,
    shelf_offset: usize,
    mascot: bool,
    devil_grin: bool,
    compact_lyrics: bool,
    glow: u32,
    dragging_glow: bool,
    transition: Option<animation::Transition>,
    content: animation::Content,
    tab_pill: animation::Pill,
    blink: animation::Blink,
    gaze: mascot::Gaze,
    grin: mascot::Grin,
    radius: i32,
    motion_enabled: bool,
    motion_mode: settings::Motion,
    focus_left: u32,
    focus_end: Option<std::time::Instant>,
    water_ml: u32,
    water_day: String,
    lyric_requests: std::sync::mpsc::Sender<lyrics::Request>,
    lyric_results: std::sync::mpsc::Receiver<lyrics::Result>,
    lyric_key: String,
    lyric_lines: Vec<lyrics::Line>,
    media: media::Snapshot,
    media_commands: std::sync::mpsc::Sender<media::Command>,
    media_updates: std::sync::mpsc::Receiver<media::Snapshot>,
    tab: usize,
    settings_return: usize,
    restart_requested: bool,
    playing: bool,
    position: u32,
    ticks: u32,
    title_font: HFONT,
    caption_font: HFONT,
    expanded: bool,
    pinned: bool,
    floating: bool,
    minimal: bool,
    tracking: bool,
    menu_open: bool,
    color: u32,
    scale: f32,
    font: HFONT,
    small_font: HFONT,
}

impl App {
    fn save(&self) {
        if let Err(error) = (Settings { motion:self.motion_mode, mascot:self.mascot, devil_grin:self.devil_grin,transparency:self.transparency,frosted:self.frosted, compact_lyrics:self.compact_lyrics,glow:self.glow, color: self.color, floating: self.floating, minimal: self.minimal }).save() {
            // Persistence failure is nonfatal; the current session keeps working.
            unsafe { OutputDebugStringW(wide(&format!("Lieve: cannot save settings: {error}")).as_ptr()); }
        }
    }
    fn px(&self, n: i32) -> i32 { (n as f32 * self.scale).round() as i32 }
    fn open_settings(&mut self) {
        if !matches!(self.tab,6|8) {self.settings_return=self.tab;}
        self.tab=6;
    }
    fn toggle_settings(&mut self) {
        if matches!(self.tab,6|8) {self.tab=self.settings_return;}else{self.open_settings();}
    }
    unsafe fn request_restart(&mut self,hwnd:HWND) {self.save();self.restart_requested=true;PostMessageW(hwnd,WM_CLOSE,0,0);}
    unsafe fn set_grin(&mut self,hwnd:HWND,enabled:bool) {
        self.devil_grin=enabled;self.grin.reset();
        if enabled {if self.motion_enabled {self.grin.start(std::time::Instant::now());}SetTimer(hwnd,6,12000,None);}
        else {KillTimer(hwnd,6);}
        self.save();self.resize(hwnd);
        self.tracking=false;SetTimer(hwnd,3,180,None);
    }
    unsafe fn refresh_motion(&mut self) {
        let mut enabled:i32=1;
        SystemParametersInfoW(SPI_GETCLIENTAREAANIMATION,0,(&mut enabled as *mut i32).cast(),0);
        self.motion_enabled=self.motion_mode.enabled(enabled!=0);
    }
    unsafe fn set_motion(&mut self,hwnd:HWND,command:u32) {
        self.motion_mode=match command {31=>settings::Motion::Full,32=>settings::Motion::Reduced,_=>settings::Motion::System};
        self.refresh_motion(); self.save(); self.resize(hwnd);
        self.tracking=false; SetTimer(hwnd,3,180,None);
    }
    unsafe fn refresh_gaze(&mut self,hwnd:HWND,now:std::time::Instant) {
        let visible=self.mascot && IsWindowVisible(hwnd)!=0;
        let idle=visible && !self.expanded && !self.minimal && !(self.is_playing() && self.compact_lyrics);
        let mut pose=mascot::Pose::default();
        if self.motion_enabled && idle {
            let mut cursor=POINT{x:0,y:0};
            let mut monitor=std::mem::zeroed::<MONITORINFO>();
            monitor.cbSize=std::mem::size_of::<MONITORINFO>() as u32;
            if GetCursorPos(&mut cursor)!=0 && GetMonitorInfoW(MonitorFromWindow(hwnd,MONITOR_DEFAULTTOPRIMARY),&mut monitor)!=0 {
                let mut input=LASTINPUTINFO {cbSize:std::mem::size_of::<LASTINPUTINFO>() as u32,dwTime:0};
                let age=if GetLastInputInfo(&mut input)!=0 {mascot::idle_millis(GetTickCount(),input.dwTime)}else{0};
                let r=monitor.rcMonitor;
                pose=mascot::Pose::from_cursor((cursor.x,cursor.y),(r.left,r.top,r.right,r.bottom),age);
            }
            // Cursor polling is separate from painting: stationary input never
            // restarts the 600 ms tween or invalidates the window.
            SetTimer(hwnd,5,if pose.lid<1. {500}else{50},None);
        } else {KillTimer(hwnd,5);}
        if self.gaze.retarget(pose,now,self.motion_enabled && visible) {
            self.clock.start(hwnd);
        }
    }
    fn size(&self) -> (i32, i32) {
        if self.expanded { (layout::PANEL_W, layout::PANEL_H) }
        else if self.osd.visible(std::time::Instant::now()) {(300,56)}
        else if self.is_playing() && self.compact_lyrics { (330, 38) }
        else if self.minimal { (128, 8) }
        else if self.mascot && self.devil_grin {(144,48)}
        else { (144, 36) }
    }

    unsafe fn resize(&mut self, hwnd: HWND) {
        let now=std::time::Instant::now();
        let motion=self.motion_enabled && IsWindowVisible(hwnd)!=0;
        // Keep the layered glass mode for the whole shell motion. Switching it
        // mid-flight visibly flashed, so drop it only at a fully settled idle.
        let glass_visible=self.expanded || self.osd.visible(now) || (self.is_playing() && self.compact_lyrics)
            || self.transition.is_some() || !self.content.sample(now).1 || !self.tab_pill.sample(now).1;
        if self.glass.configure(hwnd,glass_visible,self.frosted && self.transparency>0) {self.region_shape=None;}
        self.refresh_gaze(hwnd,now);
        let stage=if self.expanded {animation::Stage::Panel}
            else if self.osd.visible(now) {animation::Stage::Osd}
            else if self.is_playing() && self.compact_lyrics {animation::Stage::Compact}
            else {animation::Stage::Idle};
        self.content.retarget(stage,now,motion);
        if let Some(slot)=layout::tab_slot(self.tab) {
            self.tab_pill.retarget((layout::TAB_START+slot as i32*layout::TAB_STEP) as f64,now,motion);
        } else if !motion {self.tab_pill.retarget(self.tab_pill.to,now,false);}
        let (w, h) = self.size();
        let (w, h) = (self.px(w), self.px(h));
        let mut monitor = std::mem::zeroed::<MONITORINFO>();
        monitor.cbSize = std::mem::size_of::<MONITORINFO>() as u32;
        GetMonitorInfoW(MonitorFromWindow(hwnd, MONITOR_DEFAULTTOPRIMARY), &mut monitor);
        let mut old = std::mem::zeroed::<RECT>();
        GetWindowRect(hwnd, &mut old);
        let x = if self.floating { old.left.max(monitor.rcWork.left).min((monitor.rcWork.right - w).max(monitor.rcWork.left)) }
            else { monitor.rcMonitor.left + (monitor.rcMonitor.right - monitor.rcMonitor.left - w) / 2 };
        let y = if self.floating { old.top.max(monitor.rcWork.top + self.px(16)).min((monitor.rcWork.bottom - h).max(monitor.rcWork.top)) }
            else { monitor.rcMonitor.top };
        let radius=self.px(if self.floating {26}else if self.expanded {22}else if self.minimal {6}else{20});
        let target = animation::Frame { x,y,w,h,radius };
        let from = animation::Frame { x:old.left,y:old.top,w:old.right-old.left,h:old.bottom-old.top,radius:self.radius };
        if !motion {
            self.transition=None; self.apply_frame(hwnd,target);
        } else if !self.transition.as_ref().map(|t| t.to==target).unwrap_or(false) && from!=target {
            let velocity=self.transition.as_ref().map(|t|t.sample(now).1).unwrap_or([0.;5]);
            self.transition=Some(animation::Transition {from,to:target,velocity,started:now});
        } else if self.region_shape.is_none() {self.apply_frame(hwnd,from);
        }
        if self.transition.is_some() || !self.content.sample(now).1 || !self.tab_pill.sample(now).1
            || (motion && (!self.blink.sample(now).1 || !self.gaze.sample(now).1 || (self.devil_grin && !self.grin.sample(now).1) || self.osd.moving(now))) {
            self.clock.start(hwnd);
        } else {self.clock.stop(hwnd);}
        InvalidateRect(hwnd,null(),0);
    }

    unsafe fn apply_frame(&mut self, hwnd: HWND, frame: animation::Frame) {
        let animation::Frame { x,y,w,h,radius } = frame;
        self.radius=radius;
        let mut previous=std::mem::zeroed::<RECT>(); GetWindowRect(hwnd,&mut previous);
        if previous.left!=x || previous.top!=y || previous.right-previous.left!=w || previous.bottom-previous.top!=h {
            SetWindowPos(hwnd, HWND_TOPMOST, x, y, w, h, SWP_NOACTIVATE);
        }
        if self.region_shape!=Some((w,h,radius)) {
            let glass=self.glass.active && !self.glass.failed.get();
            let region = if glass {CreateRectRgn(0,0,w,h)}else{CreateRoundRectRgn(0, 0, w + 1, h + 1, radius*2, radius*2)};
            if !self.floating && !glass {
                let top = CreateRectRgn(0, 0, w, radius.min(h/2));
                CombineRgn(region, region, top, RGN_OR); DeleteObject(top);
            }
            if SetWindowRgn(hwnd, region, 0) == 0 {DeleteObject(region);}
            else {self.region_shape=Some((w,h,radius));}
        }
        InvalidateRect(hwnd, null(), 0);
    }

    unsafe fn text(&self, dc: HDC, text: &str, x: i32, y: i32, color: u32, small: bool) {
        SelectObject(dc, if small { self.small_font } else { self.font });
        SetTextColor(dc, color);
        let text = wide(text);
        TextOutW(dc, self.px(x), self.px(y), text.as_ptr(), text.len() as i32 - 1);
    }

    unsafe fn eyes(&self, dc: HDC, x: i32, y: i32) {
        let destination=dc;
        let (width,height)=(self.px(92),self.px(64));
        let mut surface=self.eye_surface.borrow_mut();
        if !surface.ensure(dc,width*4,height*4) {return;}
        let dc=surface.dc;
        let background=CreateSolidBrush(rgb(8,8,11));
        FillRect(dc,&RECT{left:0,top:0,right:width*4,bottom:height*4},background); DeleteObject(background);
        let now=std::time::Instant::now();
        let grin=if self.devil_grin {if self.motion_enabled {self.grin.sample(now).0}else{1.}}else{0.};
        let blink=if self.motion_enabled {self.blink.sample(now).0}else{1.};
        let mut pose=if self.motion_enabled {self.gaze.sample(now).0}else{mascot::Pose::default()};
        pose.lid=pose.lid*(1.-grin)+0.78*grin;pose.dx*=1.-grin;pose.dy*=1.-grin;pose.tilt=pose.tilt*(1.-grin)-1.2*grin;
        let brush = CreateSolidBrush(self.color);
        let old_brush = SelectObject(dc, brush);
        let old_pen = SelectObject(dc, GetStockObject(NULL_PEN));
        for mirror in [false, true] {
            let coords = [(0,0), (12,0), (24,8), (30,19), (16,21), (3,17), (0,0)];
            let points: Vec<POINT> = coords.iter().map(|&(a,b)| {
                let (a,b)=pose.point((if mirror {(68-a) as f64}else{a as f64},b as f64),blink);
                POINT {x:((12.+a)*self.scale as f64*4.).round() as i32,
                    y:((12.+b-9.*grin)*self.scale as f64*4.).round() as i32}
            }).collect();
            // Low-cost layered halo; no continuously running blur/render loop.
            if self.glow>0 {
                for (width,strength) in [(9,0.07),(6,0.14),(3,0.26)] {
                    let t=strength*self.glow as f64/100.;
                    let channel=|shift:u32| (8. + (((self.color>>shift)&255u32) as f64-8.)*t) as u32;
                    let halo=CreatePen(PS_SOLID,self.px(width)*4,rgb(channel(0),channel(8),channel(16)));
                    let previous=SelectObject(dc,halo);
                    BeginPath(dc); PolyBezier(dc,points.as_ptr(),points.len() as u32);
                    CloseFigure(dc); EndPath(dc); StrokePath(dc);
                    SelectObject(dc,previous); DeleteObject(halo);
                }
            }
            BeginPath(dc);
            PolyBezier(dc, points.as_ptr(), points.len() as u32);
            CloseFigure(dc);
            EndPath(dc);
            FillPath(dc);
        }
        SelectObject(dc, old_pen);
        SelectObject(dc, old_brush);
        DeleteObject(brush);
        mascot::draw_grin(dc,self.scale as f64*4.,grin);
        let mut raster=self.eye_raster.borrow_mut();surface.resolve(&mut raster,width,height,4);
        raster.draw(destination,self.px(x-12),self.px(y-12));
    }

    unsafe fn paint(&self, hwnd: HWND) {self.paint_target(hwnd,None);}
    unsafe fn paint_target(&self,hwnd:HWND,print:Option<HDC>) {
        let mut ps = std::mem::zeroed::<PAINTSTRUCT>();
        let target = print.unwrap_or_else(||BeginPaint(hwnd, &mut ps));
        let mut client=std::mem::zeroed::<RECT>(); GetClientRect(hwnd,&mut client);
        let cw=client.right.max(1); let ch=client.bottom.max(1);
        let mut surfaces=self.surfaces.borrow_mut();
        if !surfaces.output.ensure(target,cw,ch) {if print.is_none() {EndPaint(hwnd,&ps);}return;}
        let dc=surfaces.output.dc;
        let background = CreateSolidBrush(rgb(8,8,11));
        FillRect(dc,&client,background);
        let weights=self.content.sample(std::time::Instant::now()).0;
        for (stage,alpha) in weights.into_iter().enumerate() {
            if alpha<=0. {continue;}
            let (w,h)=match stage {2=>(layout::PANEL_W,layout::PANEL_H),3=>(300,56),1=>(330,38),_=>(144,if self.mascot && self.devil_grin && !self.minimal {48}else{36})};
            let (pw,ph)=(self.px(w),self.px(h));
            if !surfaces.layers[stage].ensure(target,pw,ph) {continue;}
            let layer=surfaces.layers[stage].dc;
            let rect=RECT{left:0,top:0,right:pw,bottom:ph};
            FillRect(layer,&rect,background); SetBkMode(layer,TRANSPARENT as i32);
            if stage==2 {self.panel(layer);
                if !matches!(self.tab,7|8|9) && self.osd.opacity(std::time::Instant::now())>0. {self.draw_osd(layer,70,54,260,42,self.osd.opacity(std::time::Instant::now()));}
            }
            else if stage==3 {self.draw_osd(layer,0,0,300,56,1.);}
            else if stage==1 {
                self.cover(layer,10,7,24);
                self.text(layer,&self.compact_line(),45,10,rgb(235,235,242),true);
                self.text(layer,"II",303,9,self.color,true);
            } else if !self.minimal {
                if self.mascot {let visible_h=(ch as f32/self.scale).min(h as f32) as i32;self.eyes(layer,(w-68)/2,(visible_h-20)/2);}
                else {
                    let mut time=std::mem::zeroed::<SYSTEMTIME>(); GetLocalTime(&mut time);
                    self.text(layer,&format!("{:02}:{:02}",time.wHour,time.wMinute),(w-44)/2,7,rgb(225,225,235),false);
                }
            }
            // Keep text/eyes at native scale. The growing shell clips centred
            // layers instead of stretching their bitmap to each intermediate size.
            let y=if stage==0 {-self.px((4.*(1.-alpha)).round() as i32)}else{0};
            if alpha>=0.999 {BitBlt(dc,(cw-pw)/2,y,pw,ph,layer,0,0,SRCCOPY);}
            else {AlphaBlend(dc,(cw-pw)/2,y,pw,ph,layer,0,0,pw,ph,
                BLENDFUNCTION{BlendOp:AC_SRC_OVER as u8,BlendFlags:0,SourceConstantAlpha:(alpha*255.).round() as u8,AlphaFormat:0});}
            SelectObject(layer,GetStockObject(SYSTEM_FONT));
        }
        DeleteObject(background);
        if self.glass.active && !self.glass.failed.get() && print.is_none() {
            let offset=(cw-self.px(layout::PANEL_W))/2;
            let rect=|x:i32,y:i32,w:i32,h:i32|RECT{left:offset+self.px(x),top:self.px(y),right:offset+self.px(x+w),bottom:self.px(y+h)};
            let mut opaque=Vec::new();
            let strength=weights[2];
            if strength>0.001 {
                if self.tab==0 {opaque.push(glass::Opaque{rect:rect(16,50,368,88),radius:self.px(10),strength});}
                if matches!(self.tab,0|1) {opaque.push(glass::Opaque{rect:rect(layout::MUSIC_PLAY.x+8,layout::MUSIC_PLAY.y-layout::BODY_OFFSET+8,20,20),radius:0,strength});}
                if self.tab==2 {opaque.push(glass::Opaque{rect:rect(28,164,96,16),radius:0,strength});}
                if self.tab==6 {opaque.push(glass::Opaque{rect:rect(22,78,152,38),radius:self.px(6),strength});}
            }
            if !glass::present(hwnd,&surfaces.output,&mut self.glass_raster.borrow_mut(),&mut self.glass_upload.borrow_mut(),cw,ch,self.transparency,self.radius,self.floating,&opaque) {
                self.glass.failed.set(true);let style=GetWindowLongPtrW(hwnd,GWL_EXSTYLE) as u32;
                SetWindowLongPtrW(hwnd,GWL_EXSTYLE,(style&!WS_EX_LAYERED) as isize);
                BitBlt(target,0,0,cw,ch,dc,0,0,SRCCOPY);
            }
        }else{BitBlt(target,0,0,cw,ch,dc,0,0,SRCCOPY);}
        if print.is_none() {EndPaint(hwnd, &ps);}
    }

    unsafe fn menu(&mut self, hwnd: HWND) {
        let menu = CreatePopupMenu();
        AppendMenuW(menu,MF_STRING|MF_GRAYED,0,wide(&format!("Power: {}",self.power.snapshot.label())).as_ptr());
        AppendMenuW(menu,MF_SEPARATOR,0,null());
        AppendMenuW(menu, MF_STRING, 14, wide("Appearance...").as_ptr());
        AppendMenuW(menu, MF_STRING, 16, wide("Control Center").as_ptr());
        AppendMenuW(menu,MF_STRING|if self.devil_grin {MF_CHECKED}else{0},22,wide("Devil grin").as_ptr());
        AppendMenuW(menu, MF_STRING, 15, wide("Clear shelf references").as_ptr());
        AppendMenuW(menu,MF_SEPARATOR,0,null());
        for (id,label,mode) in [(30,"Motion: follow Windows",settings::Motion::System),
            (31,"Motion: web spring",settings::Motion::Full),(32,"Motion: reduced",settings::Motion::Reduced)] {
            AppendMenuW(menu,MF_STRING|if self.motion_mode==mode {MF_CHECKED}else{0},id,wide(label).as_ptr());
        }
        AppendMenuW(menu, MF_SEPARATOR, 0, null());
        for (id, label) in [(1,"Crimson eyes"),(2,"Violet eyes"),(3,"Ice eyes"),(4,"Emerald eyes")] {
            AppendMenuW(menu, MF_STRING, id, wide(label).as_ptr());
        }
        AppendMenuW(menu, MF_STRING, 5, wide("Custom eye colour...").as_ptr());
        AppendMenuW(menu, MF_SEPARATOR, 0, null());
        for (id, label, on) in [(10,"Floating (drag header to move)",self.floating), (11,"Minimal when closed",self.minimal), (12,"Pin open",self.pinned), (13,"Demo music / compact lyrics",self.playing)] {
            AppendMenuW(menu, MF_STRING | if on { MF_CHECKED } else { 0 }, id, wide(label).as_ptr());
        }
        AppendMenuW(menu, MF_SEPARATOR, 0, null());
        AppendMenuW(menu, MF_STRING, 21, wide("Restart Lieve").as_ptr());
        AppendMenuW(menu, MF_STRING, 20, wide("Exit Lieve").as_ptr());
        let mut point = POINT { x:0, y:0 };
        GetCursorPos(&mut point);
        SetForegroundWindow(hwnd);
        let command = TrackPopupMenu(menu, TPM_RETURNCMD | TPM_RIGHTBUTTON, point.x, point.y, 0, hwnd, null());
        PostMessageW(hwnd, WM_NULL, 0, 0);
        DestroyMenu(menu);
        match command {
            30..=32 => {self.set_motion(hwnd,command as u32);return;}
            1..=4 => {
                self.color = match command { 2 => rgb(155,77,255), 3 => rgb(127,231,255), 4 => rgb(34,197,94), _ => rgb(226,54,54) };
            }
            5 => self.choose_color(hwnd),
            10 => self.floating = !self.floating,
            11 => self.minimal = !self.minimal,
            12 => { self.pinned = !self.pinned; self.expanded = self.pinned; }
            13 => self.playing = !self.playing,
            14 => { self.open_settings(); self.expanded=true; }
            15 => { self.shelf.clear(); self.shelf_offset=0; self.save_shelf(hwnd); }
            16 => {if self.tab!=7 {self.control_return=self.tab;}self.tab=7;self.expanded=true;self.pinned=true;self.system_controller.refresh();}
            20 => { PostMessageW(hwnd, WM_CLOSE, 0, 0); return; }
            21 => {self.request_restart(hwnd);return;}
            22 => {self.set_grin(hwnd,!self.devil_grin);return;}
            _ => {}
        }
        self.save();
        self.resize(hwnd);
        self.tracking = false;
        SetTimer(hwnd,3,180,None);
    }
}

// GetLocalTime is a tiny kernel32 call; keeping dependencies limited to Win32.
#[link(name = "kernel32")]
extern "system" { fn GetLocalTime(time: *mut SYSTEMTIME); }
#[link(name = "kernel32")]
extern "system" { fn GetTickCount()->u32; }
#[link(name = "kernel32")]
extern "system" { fn OutputDebugStringW(text: *const u16); }
unsafe fn initialize_runtime() {
    let _ = windows::Win32::System::WinRT::RoInitialize(windows::Win32::System::WinRT::RO_INIT_MULTITHREADED);
}

unsafe extern "system" fn wndproc(hwnd: HWND, msg: u32, wp: WPARAM, lp: LPARAM) -> LRESULT {
    if msg != 0 && msg == TASKBAR_CREATED.load(Ordering::Relaxed) {
        tray(hwnd, NIM_ADD);
        return 0;
    }
    let ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *const RefCell<App>;
    if msg == WM_NCCREATE {
        let create = &*(lp as *const CREATESTRUCTW);
        SetWindowLongPtrW(hwnd, GWLP_USERDATA, create.lpCreateParams as isize);
        return 1;
    }
    if ptr.is_null() { return DefWindowProcW(hwnd, msg, wp, lp); }
    if msg == WM_DESTROY { tray(hwnd, NIM_DELETE); for id in [1,2,3,4,5,6,7,8] { KillTimer(hwnd, id); } PostQuitMessage(0); return 0; }
    // Win32 calls can synchronously re-enter this procedure during resize/menu/drag.
    let Ok(mut app) = (&*ptr).try_borrow_mut() else {
        return DefWindowProcW(hwnd, msg, wp, lp);
    };
    match msg {
        WM_COMMAND if wp&0xffff==16 => {app.control_return=app.tab;app.tab=7;app.expanded=true;app.pinned=true;app.system_controller.refresh();app.resize(hwnd);0}
        WM_COMMAND if wp&0xffff==21 => {app.request_restart(hwnd);0}
        WM_COMMAND if wp&0xffff==22 => {let enabled=!app.devil_grin;app.set_grin(hwnd,enabled);0}
        WM_COMMAND if (30..=32).contains(&(wp as u32 & 0xffff)) => {
            app.set_motion(hwnd,wp as u32 & 0xffff); 0
        }
        WM_SETTINGCHANGE => {app.refresh_motion(); app.resize(hwnd); 0}
        WM_PAINT => { app.paint(hwnd); 0 }
        WM_PRINT|WM_PRINTCLIENT => {app.paint_target(hwnd,Some(wp as HDC));0}
        WM_ERASEBKGND => 1,
        WM_MOUSEACTIVATE => MA_NOACTIVATE as isize,
        WM_MOUSEMOVE => {
            if let Some(kind)=app.system_drag {
                let mut client=std::mem::zeroed::<RECT>();GetClientRect(hwnd,&mut client);
                let x=(lp as i16 as f32/app.scale) as i32-(((client.right-app.px(layout::PANEL_W))/2) as f32/app.scale).round() as i32;
                app.system_slider(hwnd,kind,x);return 0;
            }
            if app.dragging_glow {
                let x=(lp as i16 as f32/app.scale) as i32;
                app.set_glow_from_pointer(x);
                InvalidateRect(hwnd,null(),0);
                return 0;
            }
            KillTimer(hwnd,3);
            if !app.tracking {
                app.tracking = true;
                let mut track = TRACKMOUSEEVENT { cbSize: std::mem::size_of::<TRACKMOUSEEVENT>() as u32, dwFlags: TME_LEAVE, hwndTrack: hwnd, dwHoverTime: 0 };
                TrackMouseEvent(&mut track);
                if !app.expanded { app.expanded = true; app.resize(hwnd); }
            }
            let mut x=(lp as i16 as f32/app.scale) as i32;
            let y=((lp>>16) as i16 as f32/app.scale) as i32;
            let mut client=std::mem::zeroed::<RECT>();GetClientRect(hwnd,&mut client);
            x-=(((client.right-app.px(layout::PANEL_W))/2) as f32/app.scale).round() as i32;
            let hover=if app.expanded {layout::hover(app.tab,x,y)}else{layout::Hover::None};
            if hover!=app.hover {app.hover=hover;InvalidateRect(hwnd,null(),0);}
            0
        }
        WM_MOUSELEAVE => {
            if app.hover!=layout::Hover::None {app.hover=layout::Hover::None;InvalidateRect(hwnd,null(),0);}
            app.tracking = false;
            if !app.pinned && !app.menu_open && app.system_drag.is_none() { SetTimer(hwnd,3,180,None); }
            0
        }
        WM_LBUTTONDOWN => {
            let mut x = (lp as i16 as f32 / app.scale) as i32;
            let mut y = ((lp >> 16) as i16 as f32 / app.scale) as i32;
            if app.expanded {
                // Hit-test the same centred, unscaled layout that paint composites.
                // Invisible incoming controls must not receive accidental clicks.
                if app.content.sample(std::time::Instant::now()).0[2]<0.9 {return 0;}
                let mut client=std::mem::zeroed::<RECT>(); GetClientRect(hwnd,&mut client);
                x-=(((client.right-app.px(layout::PANEL_W))/2) as f32/app.scale).round() as i32;
                if layout::utility_hit(x,y,layout::SETTINGS_LEFT) {
                    app.toggle_settings();app.resize(hwnd);return 0;
                }
                if layout::utility_hit(x,y,layout::CONTROL_LEFT) {app.toggle_control(hwnd);return 0;}
                if (layout::NAV_TOP..layout::NAV_BOTTOM).contains(&y) && (16..296).contains(&x) {
                    let slot=((x-16)/layout::TAB_STEP).min(4) as usize;
                    app.tab=layout::TABS[slot];app.resize(hwnd);return 0;
                }
                if y<44 {
                    if app.floating {ReleaseCapture();SendMessageW(hwnd,WM_NCLBUTTONDOWN,HTCAPTION as usize,0);}
                    return 0;
                }
                if !matches!(app.tab,7|8|9) && app.osd.opacity(std::time::Instant::now())>0. && (layout::Rect{x:70,y:54,w:260,h:42}).contains(x,y) {return 0;}
                y+=layout::BODY_OFFSET;
            }
            if app.expanded && matches!(app.tab,7|9) {app.control_click(hwnd,x,y);
            } else if app.expanded && app.tab==8 {app.effects_click(hwnd,x,y);
            } else if app.expanded && app.tab==5 && y>=84 {
                app.todo_click(hwnd,x,y);
            } else if app.expanded && app.tab==4 && y>=84 {
                app.shelf_click(hwnd,x,y);
            } else if app.expanded && app.tab==6 && y>=84 {
                app.appearance_click(hwnd,x,y);
            } else if app.expanded && matches!(app.tab,0|1) {
                match layout::music_action(x,y) {
                    Some(layout::MusicAction::Lyrics)=> {app.tab=if app.tab==1 {0}else{1};app.resize(hwnd);}
                    Some(layout::MusicAction::Play)=> {
                        if app.media.available {let _=app.media_commands.send(media::Command::Toggle);}else{app.playing=!app.playing;}
                    }
                    Some(action)=> {
                        if app.media.available {let _=app.media_commands.send(if action==layout::MusicAction::Previous {media::Command::Previous}else{media::Command::Next});}
                        else {app.position=0;}
                    }
                    None if layout::MUSIC_SEEK.contains(x,y)=> {
                        let duration=if app.media.available {app.media.duration.max(1)}else{193};
                        let seconds=((x-layout::MUSIC_SEEK.x) as u32*duration/layout::MUSIC_SEEK.w as u32).min(duration.saturating_sub(1));
                        if app.media.available {let _=app.media_commands.send(media::Command::Seek(seconds));}else{app.position=seconds;}
                    }
                    _=>{}
                }
                InvalidateRect(hwnd, null(), 0);
            } else if app.expanded && app.tab == 2 && (196..228).contains(&y) && ((20..132).contains(&x) || (144..232).contains(&x)) {
                if x < 132 {
                    if app.focus_end.is_some() { app.focus_end = None; }
                    else {
                        if app.focus_left == 0 { app.focus_left = 1500; }
                        app.focus_end = Some(std::time::Instant::now() + std::time::Duration::from_secs(app.focus_left as u64));
                    }
                } else { app.focus_left = 1500; app.focus_end = None; }
                InvalidateRect(hwnd, null(), 0);
            } else if app.expanded && app.tab == 3 && (196..228).contains(&y) && ((20..132).contains(&x) || (144..256).contains(&x) || (268..380).contains(&x)) {
                if x < 132 { app.water_ml = app.water_ml.saturating_add(200); }
                else if x < 256 { app.water_ml = app.water_ml.saturating_add(400); }
                else { app.water_ml = 0; }
                if let Err(error) = water::save(&app.water_day, app.water_ml) {
                    OutputDebugStringW(wide(&format!("Lieve: cannot save water log: {error}")).as_ptr());
                }
                InvalidateRect(hwnd, null(), 0);
            } else if !app.expanded && app.floating && y < 48 {
                ReleaseCapture();
                SendMessageW(hwnd, WM_NCLBUTTONDOWN, HTCAPTION as usize, 0);
            } else if !app.expanded && y < 48 {
                app.pinned = !app.pinned;
                InvalidateRect(hwnd, null(), 0);
            }
            0
        }
        WM_LBUTTONUP if app.system_drag.is_some()=> {let kind=app.system_drag.take();ReleaseCapture();if kind==Some(system_ui::Drag::Transparency) {app.save();}app.tracking=false;SetTimer(hwnd,3,180,None);0}
        WM_LBUTTONUP if app.dragging_glow => {
            app.dragging_glow=false; ReleaseCapture(); app.save(); 0
        }
        WM_CAPTURECHANGED => {
            if app.system_drag.take()==Some(system_ui::Drag::Transparency) {app.save();}
            if app.dragging_glow {app.dragging_glow=false; app.save();}
            0
        }
        WM_DROPFILES => {
            let drop=wp as HDROP;
            let count=DragQueryFileW(drop,u32::MAX,null_mut(),0);
            for i in 0..count.min(100) {
                let len=DragQueryFileW(drop,i,null_mut(),0);
                let mut buffer=vec![0u16;len as usize+1];
                let len=DragQueryFileW(drop,i,buffer.as_mut_ptr(),buffer.len() as u32);
                use std::os::windows::ffi::OsStringExt;
                let path=std::ffi::OsString::from_wide(&buffer[..len as usize]);
                shelf::add(&mut app.shelf,std::path::PathBuf::from(path));
            }
            DragFinish(drop);
            app.tab=4; app.expanded=true; app.shelf_offset=0;
            app.save_shelf(hwnd); app.resize(hwnd); 0
        }
        WM_MOUSEWHEEL if app.expanded && app.tab==5 => {
            if ((wp>>16) as i16)<0 {app.task_offset=(app.task_offset+1).min(app.tasks.len().saturating_sub(4));}
            else {app.task_offset=app.task_offset.saturating_sub(1);}
            InvalidateRect(hwnd,null(),0); 0
        }
        WM_MOUSEWHEEL if app.expanded && app.tab==4 => {
            let delta=(wp>>16) as i16;
            if delta<0 {app.shelf_offset=(app.shelf_offset+1).min(app.shelf.len().saturating_sub(4));}
            else {app.shelf_offset=app.shelf_offset.saturating_sub(1);}
            InvalidateRect(hwnd,null(),0); 0
        }
        WM_CONTEXTMENU => {
            app.menu_open = true;
            app.menu(hwnd);
            app.menu_open = false;
            0
        }
        WM_TRAY => {
            match lp as u32 {
                WM_RBUTTONUP | WM_CONTEXTMENU => {
                    app.menu_open = true;
                    app.menu(hwnd);
                    app.menu_open = false;
                }
                WM_LBUTTONUP => {
                    app.pinned = !app.pinned;
                    app.expanded = app.pinned;
                    app.resize(hwnd);
                }
                _ => {}
            }
            0
        }
        WM_TIMER | WM_FRAME if msg==WM_FRAME || wp==2 => {
            let now=std::time::Instant::now();
            if !app.expanded && app.mascot {app.refresh_gaze(hwnd,now);}
            if let Some(t) = &app.transition {
                let (frame,_,settled)=t.sample(now);
                if settled {app.transition=None;}
                app.apply_frame(hwnd,frame);
            }
            InvalidateRect(hwnd,null(),0);
            if app.transition.is_none() && app.content.sample(now).1 && app.tab_pill.sample(now).1
                && (!app.motion_enabled || (app.blink.sample(now).1 && app.gaze.sample(now).1 && (!app.devil_grin || app.grin.sample(now).1) && !app.osd.moving(now))) {app.clock.stop(hwnd);}
            0
        }
        WM_TIMER if wp == 3 => {
            KillTimer(hwnd,3);
            let mut cursor=POINT{x:0,y:0}; GetCursorPos(&mut cursor);
            let mut rect=std::mem::zeroed::<RECT>(); GetWindowRect(hwnd,&mut rect);
            let outside=cursor.x<rect.left || cursor.x>=rect.right || cursor.y<rect.top || cursor.y>=rect.bottom;
            if outside && !app.pinned && !app.menu_open && !app.dragging_glow && app.system_drag.is_none() { app.expanded=false; app.resize(hwnd); }
            0
        }
        WM_TIMER if wp == 4 => {
            if app.motion_enabled && app.mascot && ((app.expanded && app.tab==6) || (!app.expanded && !app.minimal && !(app.is_playing() && app.compact_lyrics))) {
                app.blink.start(std::time::Instant::now()); app.clock.start(hwnd);
            }
            SetTimer(hwnd,4,app.blink.delay_ms(),None); 0
        }
        WM_TIMER if wp == 5 => {app.refresh_gaze(hwnd,std::time::Instant::now()); 0}
        WM_TIMER if wp==7 => {app.system_updates(hwnd);0}
        WM_TIMER if wp==8 => {
            if let Some(delay)=app.osd.expiry_delay(std::time::Instant::now()) {SetTimer(hwnd,8,delay,None);}
            else {KillTimer(hwnd,8);app.resize(hwnd);app.clock.start(hwnd);}0
        }
        WM_TIMER if wp == 6 => {
            if app.motion_enabled && app.devil_grin && app.mascot && ((app.expanded && app.tab==6) || (!app.expanded && !app.minimal && !(app.is_playing() && app.compact_lyrics))) {
                app.grin.start(std::time::Instant::now());app.clock.start(hwnd);
            }
            0
        }
        WM_TIMER => {
            let previous_size = app.size();
            let mut changed = false;
            if app.ticks % 30 == 0 {
                let snapshot = power::read();
                changed |= snapshot != app.power.snapshot;
                if let Some(percent) = app.power.update(snapshot) {
                    let mut notice = std::mem::zeroed::<NOTIFYICONDATAW>();
                    notice.cbSize = std::mem::size_of::<NOTIFYICONDATAW>() as u32;
                    notice.hWnd = hwnd; notice.uID = 1; notice.uFlags = NIF_INFO;
                    notice.dwInfoFlags = NIIF_WARNING;
                    let title = wide("Lieve — low battery");
                    notice.szInfoTitle[..title.len()].copy_from_slice(&title);
                    let text = wide(&format!("Battery at {percent}%. Connect your charger when you can."));
                    notice.szInfo[..text.len()].copy_from_slice(&text);
                    Shell_NotifyIconW(NIM_MODIFY, &notice);
                }
            }
            let today = water::today();
            if today != app.water_day {
                app.water_day = today;
                app.water_ml = 0;
                changed = true;
            }
            if let Some(end) = app.focus_end {
                let left = end.saturating_duration_since(std::time::Instant::now());
                app.focus_left = left.as_secs() as u32 + u32::from(left.subsec_nanos() > 0);
                changed = true;
                if app.focus_left == 0 {
                    app.focus_end = None;
                    let mut notice = std::mem::zeroed::<NOTIFYICONDATAW>();
                    notice.cbSize = std::mem::size_of::<NOTIFYICONDATAW>() as u32;
                    notice.hWnd = hwnd; notice.uID = 1; notice.uFlags = NIF_INFO;
                    let title = wide("Lieve — focus complete");
                    notice.szInfoTitle[..title.len()].copy_from_slice(&title);
                    let text = wide("Time for a five minute break.");
                    notice.szInfo[..text.len()].copy_from_slice(&text);
                    Shell_NotifyIconW(NIM_MODIFY, &notice);
                }
            }
            while let Ok(snapshot) = app.media_updates.try_recv() {
                let key = format!("{}\n{}\n{}",snapshot.title,snapshot.artist,snapshot.duration);
                if key != app.lyric_key {
                    app.lyric_key = key.clone();
                    app.lyric_lines.clear();
                    if snapshot.available && !snapshot.title.is_empty() {
                        let _ = app.lyric_requests.send(lyrics::Request { key, title:snapshot.title.clone(), artist:snapshot.artist.clone(), duration:snapshot.duration });
                    }
                }
                app.media = snapshot;
                changed = true;
            }
            while let Ok(result) = app.lyric_results.try_recv() {
                if result.key == app.lyric_key { app.lyric_lines = result.lines; changed = true; }
            }
            app.ticks = app.ticks.wrapping_add(1);
            if app.playing && !app.media.available { app.position = (app.position + 1) % 193; }
            if changed {
                if previous_size != app.size() { app.resize(hwnd); }
                else { InvalidateRect(hwnd, null(), 0); }
            }
            else if app.playing || app.ticks % 60 == 0 { InvalidateRect(hwnd, null(), 0); }
            0
        }
        WM_DISPLAYCHANGE | WM_EXITSIZEMOVE => { app.clock.monitor_changed(hwnd); app.resize(hwnd); 0 }
        WM_CLOSE => { DestroyWindow(hwnd); 0 }
        _ => DefWindowProcW(hwnd, msg, wp, lp),
    }
}

fn main() {
    unsafe {
        SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);
        TASKBAR_CREATED.store(RegisterWindowMessageW(wide("TaskbarCreated").as_ptr()), Ordering::Relaxed);
        let instance = GetModuleHandleW(null());
        let class = wide("LieveNativeIsland");
        if !FindWindowW(class.as_ptr(), null()).is_null() { return; }
        let wc = WNDCLASSW { lpfnWndProc: Some(wndproc), hInstance: instance, lpszClassName: class.as_ptr(), hCursor: LoadCursorW(null_mut(), IDC_ARROW), ..std::mem::zeroed() };
        RegisterClassW(&wc);
        let scale = GetDpiForSystem() as f32 / 96.;
        let settings = Settings::load();
        let font = |size: f32, weight| CreateFontW(-(size * scale) as i32, 0, 0, 0, weight, 0, 0, 0, DEFAULT_CHARSET as u32, 0, 0, CLEARTYPE_QUALITY as u32, 0, wide("Segoe UI").as_ptr());
        let (media_commands, media_updates) = media::start();
        let (system_controller,system_updates)=system_control::start();
        let (lyric_requests, lyric_results) = lyrics::start();
        let water_day = water::today();
        let water_ml = water::load(&water_day);
        let mut motion: i32=1;
        SystemParametersInfoW(SPI_GETCLIENTAREAANIMATION,0,(&mut motion as *mut i32).cast(),0);
        motion=i32::from(settings.motion.enabled(motion!=0));
        let app = Box::new(RefCell::new(App { tasks:todo::load(), task_offset:0, shelf:shelf::load(), shelf_offset:0, mascot:settings.mascot, compact_lyrics:settings.compact_lyrics, glow:settings.glow, dragging_glow:false, transition:None, motion_enabled:motion!=0, focus_left:1500, focus_end:None, water_ml, water_day, lyric_requests, lyric_results, lyric_key:String::new(), lyric_lines:Vec::new(), media:media::Snapshot::default(), media_commands, media_updates, tab:0, playing:false, position:66, ticks:0, title_font:font(22.,600), expanded:false, pinned:false, floating:settings.floating, minimal:settings.minimal, tracking:false, menu_open:false,
            color: settings.color,
            motion_mode: settings.motion,
            content: animation::Content::new(), tab_pill: animation::Pill::new(), radius:0,
            blink: animation::Blink::new(),
            gaze: mascot::Gaze::new(),
            grin:mascot::Grin::new(),devil_grin:settings.devil_grin,
            settings_return:0,
            restart_requested:false,
            clock:frame_clock::FrameClock::new(),surfaces:RefCell::new(render::Surfaces::new()),
            eye_surface:RefCell::new(render::Surface::new()),region_shape:None,
            eye_raster:RefCell::new(render::Raster::new()),cover_cache:RefCell::new(render::CoverCache::new()),
            shapes:RefCell::new(render::ShapeCache::new()),
            glass:glass::Glass::new(),glass_raster:RefCell::new(render::Raster::new()),glass_upload:RefCell::new(render::Surface::new()),
            transparency:settings.transparency,frosted:settings.frosted,system:system_control::Snapshot::default(),system_controller,system_updates,
            system_drag:None,osd:osd::Hud::new(),control_return:0,
            icons:RefCell::new(icons::Renderer::new()),hover:layout::Hover::None,caption_font:font(10.,600),
            power: power::Monitor::new(power::read()),
            scale, font:font(16., 600), small_font:font(12., 400) }));
        let hwnd = CreateWindowExW(WS_EX_TOPMOST | WS_EX_TOOLWINDOW, class.as_ptr(), wide("Lieve").as_ptr(), WS_POPUP,
            0, 0, 196, 36, null_mut(), null_mut(), instance, (&*app as *const RefCell<App>).cast_mut().cast());
        if hwnd.is_null() { return; }
        let focus=std::env::args().find_map(|arg|restart::Focus::parse(&arg)).unwrap_or(restart::Focus::Paused(1500))
            .restore(std::time::Instant::now(),std::time::SystemTime::now());
        app.borrow_mut().focus_left=focus.0;app.borrow_mut().focus_end=focus.1;
        app.borrow_mut().clock.monitor_changed(hwnd);
        app.borrow_mut().resize(hwnd);
        ShowWindow(hwnd, SW_SHOWNOACTIVATE);
        app.borrow_mut().refresh_gaze(hwnd,std::time::Instant::now());
        DragAcceptFiles(hwnd,1);
        tray(hwnd, NIM_ADD);
        SetTimer(hwnd, 1, 1000, None);
        SetTimer(hwnd,7,100,None);
        SetTimer(hwnd, 4, 4200, None);
        if settings.devil_grin {SetTimer(hwnd,6,12000,None);}
        let mut msg = std::mem::zeroed::<MSG>();
        'messages: loop {
            // Bound message draining so heavy input cannot starve an armed frame.
            for _ in 0..64 {
                if PeekMessageW(&mut msg,null_mut(),0,0,PM_REMOVE)==0 {break;}
                if msg.message==WM_QUIT {break 'messages;}
                TranslateMessage(&msg); DispatchMessageW(&msg);
            }
            let timer=app.borrow().clock.handle();
            let count=u32::from(!timer.is_null());
            let result=MsgWaitForMultipleObjectsEx(count,&timer,u32::MAX,QS_ALLINPUT,MWMO_INPUTAVAILABLE);
            if count==1 && result==WAIT_OBJECT_0 {
                SendMessageW(hwnd,WM_FRAME,0,0);
                // Paint after wndproc releases App's RefCell borrow, before the
                // next deadline. Avoid WM_PAINT being coalesced behind input.
                UpdateWindow(hwnd);
                app.borrow_mut().clock.complete(hwnd);
            } else if result==WAIT_FAILED {break;}
        }
        DeleteObject(app.borrow().font);
        let resume={let state=app.borrow();state.restart_requested.then(||restart::Focus::capture(state.focus_left,state.focus_end,std::time::Instant::now(),std::time::SystemTime::now()))};
        DeleteObject(app.borrow().small_font);
        DeleteObject(app.borrow().title_font);
        DeleteObject(app.borrow().caption_font);
        drop(app);
        if let Some(focus)=resume {if let Err(error)=restart::launch(focus) {
            MessageBoxW(null_mut(),wide(&format!("Could not restart Lieve: {error}")).as_ptr(),wide("Lieve").as_ptr(),MB_OK|MB_ICONERROR);
        }}
    }
}
