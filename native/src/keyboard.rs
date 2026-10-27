use super::*;
pub const HOTKEY_ID:i32=0x4c49;
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum Target {Tab(usize),Bluetooth,Control,Settings,Back,System,Glass,Spectrum,Startup,Fullscreen,SpectrumToggle,SpectrumColor,Sensitivity,RefreshBluetooth,WindowsBluetooth,PreviewPlug,PreviewUnplug}
pub struct Navigation {pub visible:bool,pub target:Target,pub ignore_hover:bool,closed_pointer:POINT,session:bool,previous:HWND,previous_pid:u32}
impl Navigation {
    pub fn new()->Self {Self{visible:false,target:Target::Tab(0),ignore_hover:false,closed_pointer:POINT{x:0,y:0},session:false,previous:null_mut(),previous_pid:0}}
    pub fn selected(tab:usize)->Target {
        layout::tab_slot(tab).map(Target::Tab).unwrap_or(if tab==9 {Target::Bluetooth}else if tab==7 {Target::Control}else{Target::Settings})
    }
    fn targets(tab:usize)->Vec<Target> {
        let mut targets=(0..5).map(Target::Tab).chain([Target::Bluetooth,Target::Control,Target::Settings]).collect::<Vec<_>>();
        targets.extend_from_slice(match tab {
            6=>&[Target::System,Target::Glass],
            10=>&[Target::Back,Target::Spectrum,Target::Startup,Target::Fullscreen],
            11=>&[Target::Back,Target::SpectrumToggle,Target::SpectrumColor,Target::Sensitivity],
            9=>&[Target::Back,Target::RefreshBluetooth,Target::WindowsBluetooth],
            12=>&[Target::Back,Target::PreviewPlug,Target::PreviewUnplug],
            _=>&[],
        });targets
    }
    pub fn cycle(&mut self,tab:usize,backward:bool) {
        if !self.visible {self.target=Self::selected(tab);}
        let targets=Self::targets(tab);let index=targets.iter().position(|t|*t==self.target).unwrap_or(0);
        self.target=targets[(index+if backward {targets.len()-1}else{1})%targets.len()];self.visible=true;
    }
    pub fn blur(&mut self) {self.visible=false;self.session=false;self.previous=null_mut();self.previous_pid=0;}
    fn pointer_moved(&mut self,p:POINT)->bool {
        if self.ignore_hover && (p.x!=self.closed_pointer.x || p.y!=self.closed_pointer.y) {self.ignore_hover=false;}
        !self.ignore_hover
    }
    pub unsafe fn allow_hover(&mut self)->bool {
        if !self.ignore_hover {return true;}
        let mut p=POINT{x:0,y:0};GetCursorPos(&mut p)!=0 && self.pointer_moved(p)
    }
    pub unsafe fn release(&mut self,hwnd:HWND) {
        self.visible=false;
        let mut pid=0;if !self.previous.is_null() {GetWindowThreadProcessId(self.previous,&mut pid);}
        if self.session && GetForegroundWindow()==hwnd && !self.previous.is_null() && pid==self.previous_pid && IsWindow(self.previous)!=0 && IsWindowVisible(self.previous)!=0 && IsIconic(self.previous)==0 {
            SetForegroundWindow(self.previous);
        }
        self.blur();
    }
}
pub fn focus_rect(target:Target)->layout::Rect {
    let body=|x,y,w,h|layout::Rect{x,y:y-layout::BODY_OFFSET,w,h};
    match target {
        Target::Tab(i)=>layout::Rect{x:layout::TAB_START+i as i32*layout::TAB_STEP,y:12,w:layout::TAB_PILL_W,h:24},
        Target::Control=>layout::Rect{x:layout::CONTROL_LEFT,y:10,w:32,h:28},
        Target::Bluetooth=>layout::Rect{x:layout::BLUETOOTH_LEFT,y:10,w:32,h:28},
        Target::Settings=>layout::Rect{x:layout::SETTINGS_LEFT,y:10,w:32,h:28},
        Target::Back=>body(16,88,112,24),Target::System=>body(204,92,88,18),Target::Glass=>body(304,92,72,18),
        Target::Spectrum=>body(264,88,120,24),Target::Startup=>body(16,124,368,42),Target::Fullscreen=>body(16,176,368,42),
        Target::SpectrumToggle=>body(16,120,368,36),Target::SpectrumColor=>body(220,168,152,24),Target::Sensitivity=>body(24,217,352,24),
        Target::RefreshBluetooth=>body(300,88,84,24),Target::WindowsBluetooth=>body(16,242,234,16),
        Target::PreviewPlug=>body(20,210,176,30),Target::PreviewUnplug=>body(204,210,176,30),
    }
}
impl App {
    pub(super) unsafe fn toggle_keyboard_panel(&mut self,hwnd:HWND) {
        self.refresh_visibility(hwnd,true);if self.hidden_fullscreen {return;}
        if self.expanded && self.navigation.session && GetForegroundWindow()==hwnd {self.close_keyboard_panel(hwnd);return;}
        let previous=GetForegroundWindow();if previous!=hwnd {self.navigation.previous=previous;GetWindowThreadProcessId(previous,&mut self.navigation.previous_pid);}
        self.navigation.session=true;self.navigation.visible=true;self.navigation.ignore_hover=false;
        self.navigation.target=Navigation::selected(self.tab);self.expanded=true;self.pinned=true;
        KillTimer(hwnd,3);self.resize(hwnd);SetForegroundWindow(hwnd);SetFocus(hwnd);
    }
    pub(super) unsafe fn close_keyboard_panel(&mut self,hwnd:HWND) {
        let kind=self.system_drag.take();if matches!(kind,Some(system_ui::Drag::Transparency|system_ui::Drag::Sensitivity)) || self.dragging_glow {self.save();}
        self.dragging_glow=false;ReleaseCapture();KillTimer(hwnd,3);
        let mut cursor=POINT{x:0,y:0};let mut bounds=std::mem::zeroed::<RECT>();
        self.navigation.ignore_hover=GetCursorPos(&mut cursor)!=0 && GetWindowRect(hwnd,&mut bounds)!=0
            && cursor.x>=bounds.left && cursor.x<bounds.right && cursor.y>=bounds.top && cursor.y<bounds.bottom;
        self.navigation.closed_pointer=cursor;
        self.expanded=false;self.pinned=false;self.tracking=false;self.resize(hwnd);
    }
    pub(super) unsafe fn keyboard_action(&mut self,hwnd:HWND,target:Target) {
        match target {
            Target::Tab(i)=>if let Some(tab)=layout::TABS.get(i) {self.tab=*tab;self.resize(hwnd);},
            Target::Control=>self.toggle_control(hwnd),Target::Settings=>{self.toggle_settings();self.resize(hwnd);},
            Target::Bluetooth=>self.toggle_bluetooth(hwnd),
            Target::Back=>{self.tab=if self.tab==9 {self.bluetooth_return}else if matches!(self.tab,11|12) {10}else{6};self.resize(hwnd);},
            Target::System=>{self.tab=10;self.resize(hwnd);},Target::Glass=>{self.tab=8;self.resize(hwnd);},
            Target::Spectrum=>{self.tab=11;self.resize(hwnd);},
            Target::Startup=>self.toggle_autostart(hwnd),Target::Fullscreen=>self.toggle_fullscreen_hide(hwnd),
            Target::SpectrumToggle=>{self.spectrum_preferences.enabled=!self.spectrum_preferences.enabled;self.spectrum_settings_changed(hwnd);},
            Target::SpectrumColor=>{self.spectrum_preferences.white=!self.spectrum_preferences.white;self.spectrum_settings_changed(hwnd);},
            Target::Sensitivity=>{},
            Target::RefreshBluetooth=>self.system_controller.refresh_bluetooth(),
            Target::WindowsBluetooth=>self.open_windows_settings(hwnd,"ms-settings:bluetooth"),
            Target::PreviewPlug|Target::PreviewUnplug=>self.show_power(hwnd,if target==Target::PreviewPlug {osd::Kind::PowerConnected}else{osd::Kind::PowerDisconnected},self.power.snapshot,true),
        }
        self.navigation.target=if Navigation::targets(self.tab).contains(&target) {target}else{Navigation::selected(self.tab)};
        InvalidateRect(hwnd,null(),0);
    }
    pub(super) unsafe fn keyboard_key(&mut self,hwnd:HWND,key:u16,repeated:bool)->bool {
        if self.hidden_fullscreen || !self.expanded || GetForegroundWindow()!=hwnd {return false;}
        let control=GetKeyState(VK_CONTROL as i32)<0;let alt=GetKeyState(VK_MENU as i32)<0;
        if key==VK_ESCAPE {self.close_keyboard_panel(hwnd);return true;}
        if control && !alt && (0x31..=0x35).contains(&key) {
            self.navigation.visible=true;self.keyboard_action(hwnd,Target::Tab((key-0x31) as usize));return true;
        }
        if self.navigation.visible && self.navigation.target==Target::Sensitivity && matches!(key,VK_LEFT|VK_RIGHT|VK_HOME|VK_END) {
            self.spectrum_preferences.sensitivity=match key {VK_HOME=>50,VK_END=>200,VK_LEFT=>self.spectrum_preferences.sensitivity.saturating_sub(5).max(50),_=>self.spectrum_preferences.sensitivity.saturating_add(5).min(200)};
            self.spectrum_settings_changed(hwnd);return true;
        }
        if self.navigation.visible && self.navigation.target==Target::SpectrumColor && matches!(key,VK_LEFT|VK_RIGHT) {
            self.spectrum_preferences.white=key==VK_RIGHT;self.spectrum_settings_changed(hwnd);return true;
        }
        if matches!(key,VK_TAB|VK_LEFT|VK_RIGHT) {
            self.navigation.cycle(self.tab,key==VK_LEFT || (key==VK_TAB && GetKeyState(VK_SHIFT as i32)<0));InvalidateRect(hwnd,null(),0);return true;
        }
        if self.navigation.visible && !control && !alt && matches!(key,VK_RETURN|VK_SPACE) {
            if !repeated {self.keyboard_action(hwnd,self.navigation.target);}return true;
        }false
    }
    pub(super) unsafe fn keyboard_focus(&self,dc:HDC) {
        if !self.navigation.visible {return;}
        let r=focus_rect(self.navigation.target);let brush=CreateSolidBrush(rgb(223,223,237));
        FrameRect(dc,&RECT{left:self.px(r.x),top:self.px(r.y),right:self.px(r.x+r.w),bottom:self.px(r.y+r.h)},brush);DeleteObject(brush);
    }
}
#[cfg(test)] mod tests {
    use super::*;
    #[test] fn focus_wraps_and_new_settings_controls_are_reachable() {
        let mut nav=Navigation::new();nav.visible=true;nav.cycle(0,true);assert_eq!(nav.target,Target::Settings);
        nav.cycle(11,false);assert_eq!(nav.target,Target::Back);
        nav.cycle(11,false);assert_eq!(nav.target,Target::SpectrumToggle);
        nav.cycle(11,false);assert_eq!(nav.target,Target::SpectrumColor);
        nav.cycle(11,false);assert_eq!(nav.target,Target::Sensitivity);
        nav.cycle(11,false);assert_eq!(nav.target,Target::Tab(0));
        for tab in [0,6,10,11] {for target in Navigation::targets(tab) {let r=focus_rect(target);assert!(r.x>=0 && r.y>=0 && r.x+r.w<=layout::PANEL_W && r.y+r.h<=layout::PANEL_H);}}
    }
    #[test] fn geometry_generated_mouse_messages_do_not_reopen_a_keyboard_closed_panel() {
        let mut nav=Navigation::new();nav.ignore_hover=true;nav.closed_pointer=POINT{x:960,y:20};
        assert!(!nav.pointer_moved(POINT{x:960,y:20}));nav.blur();assert!(!nav.pointer_moved(POINT{x:960,y:20}));
        assert!(nav.pointer_moved(POINT{x:961,y:20}));assert!(!nav.ignore_hover);
    }
}
