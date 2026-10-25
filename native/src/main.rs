#![windows_subsystem = "windows"]

mod settings;
mod visual;
mod media;
mod lyrics;
mod water;
mod animation;
mod appearance;
mod shelf;
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
    shelf: Vec<std::path::PathBuf>,
    shelf_offset: usize,
    mascot: bool,
    compact_lyrics: bool,
    glow: u32,
    dragging_glow: bool,
    transition: Option<animation::Transition>,
    motion_enabled: bool,
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
    playing: bool,
    position: u32,
    ticks: u32,
    title_font: HFONT,
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
        if let Err(error) = (Settings { mascot:self.mascot, compact_lyrics:self.compact_lyrics, glow:self.glow, color: self.color, floating: self.floating, minimal: self.minimal }).save() {
            // Persistence failure is nonfatal; the current session keeps working.
            unsafe { OutputDebugStringW(wide(&format!("Lieve: cannot save settings: {error}")).as_ptr()); }
        }
    }
    fn px(&self, n: i32) -> i32 { (n as f32 * self.scale).round() as i32 }
    fn size(&self) -> (i32, i32) {
        if self.expanded { (470, 322) }
        else if self.is_playing() && self.compact_lyrics { (330, 38) }
        else if self.minimal { (128, 8) }
        else { (144, 36) }
    }

    unsafe fn resize(&mut self, hwnd: HWND) {
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
        let target = animation::Frame { x,y,w,h };
        let from = animation::Frame { x:old.left,y:old.top,w:old.right-old.left,h:old.bottom-old.top };
        if self.transition.as_ref().map(|t| t.to == target).unwrap_or(false) { return; }
        if self.motion_enabled && IsWindowVisible(hwnd) != 0 && from != target {
            self.transition = Some(animation::Transition { from, to:target, started:std::time::Instant::now() });
            SetTimer(hwnd, 2, 16, None);
        } else {
            self.transition = None;
            KillTimer(hwnd,2);
            self.apply_frame(hwnd,target);
        }
    }

    unsafe fn apply_frame(&self, hwnd: HWND, frame: animation::Frame) {
        let animation::Frame { x,y,w,h } = frame;
        SetWindowPos(hwnd, HWND_TOPMOST, x, y, w, h, SWP_NOACTIVATE);
        let region = CreateRoundRectRgn(0, 0, w + 1, h + 1, self.px(30), self.px(30));
        if !self.floating {
            let top = CreateRectRgn(0, 0, w, self.px(15));
            CombineRgn(region, region, top, RGN_OR);
            DeleteObject(top);
        }
        if SetWindowRgn(hwnd, region, 1) == 0 { DeleteObject(region); }
        InvalidateRect(hwnd, null(), 0);
    }

    unsafe fn text(&self, dc: HDC, text: &str, x: i32, y: i32, color: u32, small: bool) {
        SelectObject(dc, if small { self.small_font } else { self.font });
        SetTextColor(dc, color);
        let text = wide(text);
        TextOutW(dc, self.px(x), self.px(y), text.as_ptr(), text.len() as i32 - 1);
    }

    unsafe fn eyes(&self, dc: HDC, x: i32, y: i32) {
        let brush = CreateSolidBrush(self.color);
        let old_brush = SelectObject(dc, brush);
        let old_pen = SelectObject(dc, GetStockObject(NULL_PEN));
        for mirror in [false, true] {
            let coords = [(0,0), (12,0), (24,8), (30,19), (16,21), (3,17), (0,0)];
            let points: Vec<POINT> = coords.iter().map(|&(a,b)| POINT {
                x: self.px(x + if mirror { 68-a } else { a }), y: self.px(y+b),
            }).collect();
            // Low-cost layered halo; no continuously running blur/render loop.
            if self.glow>0 {
                for (width,strength) in [(9,0.07),(6,0.14),(3,0.26)] {
                    let t=strength*self.glow as f64/100.;
                    let channel=|shift:u32| (8. + (((self.color>>shift)&255u32) as f64-8.)*t) as u32;
                    let halo=CreatePen(PS_SOLID,self.px(width),rgb(channel(0),channel(8),channel(16)));
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
    }

    unsafe fn paint(&self, hwnd: HWND) {
        let mut ps = std::mem::zeroed::<PAINTSTRUCT>();
        let target = BeginPaint(hwnd, &mut ps);
        let (w, h) = if !self.expanded && self.transition.as_ref().map(|t|t.from.h > self.px(100)).unwrap_or(false) {
            (470,322)
        } else { self.size() };
        let dc = CreateCompatibleDC(target);
        let bitmap = CreateCompatibleBitmap(target, self.px(w), self.px(h));
        let old_bitmap = SelectObject(dc, bitmap);
        let background = CreateSolidBrush(rgb(8,8,11));
        let rect = RECT { left: 0, top: 0, right: self.px(w), bottom: self.px(h) };
        FillRect(dc, &rect, background);
        DeleteObject(background);
        SetBkMode(dc, TRANSPARENT as i32);
        if self.expanded || h == 322 {
            self.panel(dc);
        } else if self.is_playing() && self.compact_lyrics {
            self.cover(dc, 10, 7, 24);
            self.text(dc, &self.compact_line(), 45, 10, rgb(235,235,242), true);
            self.text(dc, "II", 303, 9, self.color, true);
        } else if !self.minimal {
            if self.mascot { self.eyes(dc, (w-68)/2, (h-20)/2); }
            else {
                let mut time=std::mem::zeroed::<SYSTEMTIME>(); GetLocalTime(&mut time);
                self.text(dc,&format!("{:02}:{:02}",time.wHour,time.wMinute),(w-44)/2,7,rgb(225,225,235),false);
            }
        }
        let mut client=std::mem::zeroed::<RECT>();
        GetClientRect(hwnd,&mut client);
        if client.right == self.px(w) && client.bottom == self.px(h) {
            BitBlt(target, 0, 0, client.right, client.bottom, dc, 0, 0, SRCCOPY);
        } else {
            SetStretchBltMode(target, HALFTONE);
            SetBrushOrgEx(target,0,0,null_mut());
            StretchBlt(target,0,0,client.right,client.bottom,dc,0,0,self.px(w),self.px(h),SRCCOPY);
        }
        SelectObject(dc, old_bitmap);
        DeleteObject(bitmap);
        DeleteDC(dc);
        EndPaint(hwnd, &ps);
    }

    unsafe fn menu(&mut self, hwnd: HWND) {
        let menu = CreatePopupMenu();
        AppendMenuW(menu, MF_STRING, 14, wide("Appearance...").as_ptr());
        AppendMenuW(menu, MF_STRING, 15, wide("Clear shelf references").as_ptr());
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
        AppendMenuW(menu, MF_STRING, 20, wide("Exit Lieve").as_ptr());
        let mut point = POINT { x:0, y:0 };
        GetCursorPos(&mut point);
        SetForegroundWindow(hwnd);
        let command = TrackPopupMenu(menu, TPM_RETURNCMD | TPM_RIGHTBUTTON, point.x, point.y, 0, hwnd, null());
        PostMessageW(hwnd, WM_NULL, 0, 0);
        DestroyMenu(menu);
        match command {
            1..=4 => {
                self.color = match command { 2 => rgb(155,77,255), 3 => rgb(127,231,255), 4 => rgb(34,197,94), _ => rgb(226,54,54) };
            }
            5 => self.choose_color(hwnd),
            10 => self.floating = !self.floating,
            11 => self.minimal = !self.minimal,
            12 => { self.pinned = !self.pinned; self.expanded = self.pinned; }
            13 => self.playing = !self.playing,
            14 => { self.tab=6; self.expanded=true; }
            15 => { self.shelf.clear(); self.shelf_offset=0; self.save_shelf(hwnd); }
            20 => { PostMessageW(hwnd, WM_CLOSE, 0, 0); return; }
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
    if msg == WM_DESTROY { tray(hwnd, NIM_DELETE); for id in [1,2,3] { KillTimer(hwnd, id); } PostQuitMessage(0); return 0; }
    // Win32 calls can synchronously re-enter this procedure during resize/menu/drag.
    let Ok(mut app) = (&*ptr).try_borrow_mut() else {
        return DefWindowProcW(hwnd, msg, wp, lp);
    };
    match msg {
        WM_PAINT => { app.paint(hwnd); 0 }
        WM_ERASEBKGND => 1,
        WM_MOUSEACTIVATE => MA_NOACTIVATE as isize,
        WM_MOUSEMOVE => {
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
            0
        }
        WM_MOUSELEAVE => {
            app.tracking = false;
            if !app.pinned && !app.menu_open { SetTimer(hwnd,3,180,None); }
            0
        }
        WM_LBUTTONDOWN => {
            let x = (lp as i16 as f32 / app.scale) as i32;
            let y = ((lp >> 16) as i16 as f32 / app.scale) as i32;
            if app.expanded && app.tab==4 && y>=96 {
                app.shelf_click(hwnd,x,y);
            } else if app.expanded && app.tab==6 && y>=96 {
                app.appearance_click(hwnd,x,y);
            } else if app.expanded && (50..85).contains(&y) && (12..458).contains(&x) {
                app.tab = ((x - 12) / 74).min(5) as usize;
                InvalidateRect(hwnd, null(), 0);
            } else if app.expanded && app.tab == 0 && (240..282).contains(&y) && (170..205).contains(&x) {
                if app.media.available { let _ = app.media_commands.send(media::Command::Toggle); }
                else { app.playing = !app.playing; }
                InvalidateRect(hwnd, null(), 0);
            } else if app.expanded && app.tab == 0 && (240..282).contains(&y) && ((135..170).contains(&x) || (210..245).contains(&x)) {
                if app.media.available {
                    let _ = app.media_commands.send(if x < 170 { media::Command::Previous } else { media::Command::Next });
                } else { app.position = 0; }
                InvalidateRect(hwnd, null(), 0);
            } else if app.expanded && app.tab == 0 && (198..223).contains(&y) && (138..453).contains(&x) {
                let duration = if app.media.available { app.media.duration } else { 193 };
                let seconds = ((x-138) as u32*duration/315).min(duration.saturating_sub(1));
                if app.media.available { let _ = app.media_commands.send(media::Command::Seek(seconds)); }
                else { app.position = seconds; }
                InvalidateRect(hwnd, null(), 0);
            } else if app.expanded && app.tab == 2 && (235..280).contains(&y) {
                if x < 150 {
                    if app.focus_end.is_some() { app.focus_end = None; }
                    else {
                        if app.focus_left == 0 { app.focus_left = 1500; }
                        app.focus_end = Some(std::time::Instant::now() + std::time::Duration::from_secs(app.focus_left as u64));
                    }
                } else { app.focus_left = 1500; app.focus_end = None; }
                InvalidateRect(hwnd, null(), 0);
            } else if app.expanded && app.tab == 3 && (235..280).contains(&y) {
                if x < 155 { app.water_ml = app.water_ml.saturating_add(200); }
                else if x < 280 { app.water_ml = app.water_ml.saturating_add(400); }
                else { app.water_ml = 0; }
                if let Err(error) = water::save(&app.water_day, app.water_ml) {
                    OutputDebugStringW(wide(&format!("Lieve: cannot save water log: {error}")).as_ptr());
                }
                InvalidateRect(hwnd, null(), 0);
            } else if app.expanded && x >= 420 && y < 45 {
                app.tab=if app.tab==6 {0} else {6};
                InvalidateRect(hwnd,null(),0);
            } else if app.expanded && x >= 385 && y < 45 {
                app.pinned = !app.pinned;
                InvalidateRect(hwnd, null(), 0);
            } else if app.floating && y < 48 {
                ReleaseCapture();
                SendMessageW(hwnd, WM_NCLBUTTONDOWN, HTCAPTION as usize, 0);
            } else if y < 48 {
                app.pinned = !app.pinned;
                InvalidateRect(hwnd, null(), 0);
            }
            0
        }
        WM_LBUTTONUP if app.dragging_glow => {
            app.dragging_glow=false; ReleaseCapture(); app.save(); 0
        }
        WM_CAPTURECHANGED => {
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
        WM_TIMER if wp == 2 => {
            if let Some(t) = &app.transition {
                let progress=t.started.elapsed().as_secs_f64()/0.26;
                let frame=animation::interpolate(t.from,t.to,progress);
                if progress>=1. { app.transition=None; KillTimer(hwnd,2); }
                app.apply_frame(hwnd,frame);
            } else { KillTimer(hwnd,2); }
            0
        }
        WM_TIMER if wp == 3 => {
            KillTimer(hwnd,3);
            let mut cursor=POINT{x:0,y:0}; GetCursorPos(&mut cursor);
            let mut rect=std::mem::zeroed::<RECT>(); GetWindowRect(hwnd,&mut rect);
            let outside=cursor.x<rect.left || cursor.x>=rect.right || cursor.y<rect.top || cursor.y>=rect.bottom;
            if outside && !app.pinned && !app.menu_open && !app.dragging_glow { app.expanded=false; app.resize(hwnd); }
            0
        }
        WM_TIMER => {
            let previous_size = app.size();
            let mut changed = false;
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
        WM_DISPLAYCHANGE => { app.resize(hwnd); 0 }
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
        let (lyric_requests, lyric_results) = lyrics::start();
        let water_day = water::today();
        let water_ml = water::load(&water_day);
        let mut motion: i32=1;
        SystemParametersInfoW(SPI_GETCLIENTAREAANIMATION,0,(&mut motion as *mut i32).cast(),0);
        let app = Box::new(RefCell::new(App { shelf:shelf::load(), shelf_offset:0, mascot:settings.mascot, compact_lyrics:settings.compact_lyrics, glow:settings.glow, dragging_glow:false, transition:None, motion_enabled:motion!=0, focus_left:1500, focus_end:None, water_ml, water_day, lyric_requests, lyric_results, lyric_key:String::new(), lyric_lines:Vec::new(), media:media::Snapshot::default(), media_commands, media_updates, tab:0, playing:false, position:66, ticks:0, title_font:font(22.,600), expanded:false, pinned:false, floating:settings.floating, minimal:settings.minimal, tracking:false, menu_open:false,
            color: settings.color,
            scale, font:font(16., 600), small_font:font(12., 400) }));
        let hwnd = CreateWindowExW(WS_EX_TOPMOST | WS_EX_TOOLWINDOW, class.as_ptr(), wide("Lieve").as_ptr(), WS_POPUP,
            0, 0, 196, 36, null_mut(), null_mut(), instance, (&*app as *const RefCell<App>).cast_mut().cast());
        if hwnd.is_null() { return; }
        app.borrow_mut().resize(hwnd);
        ShowWindow(hwnd, SW_SHOWNOACTIVATE);
        DragAcceptFiles(hwnd,1);
        tray(hwnd, NIM_ADD);
        SetTimer(hwnd, 1, 1000, None);
        let mut msg = std::mem::zeroed::<MSG>();
        while GetMessageW(&mut msg, null_mut(), 0, 0) > 0 { TranslateMessage(&msg); DispatchMessageW(&msg); }
        DeleteObject(app.borrow().font);
        DeleteObject(app.borrow().small_font);
        DeleteObject(app.borrow().title_font);
    }
}
