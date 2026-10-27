// Logical pixels, shared by panel rendering, spring targets and pointer mapping.
pub const PANEL_W:i32=400;
pub const PANEL_H:i32=220;
pub const BODY_OFFSET:i32=40;
pub const TAB_START:i32=18;
pub const TAB_STEP:i32=48;
pub const TAB_PILL_W:i32=44;
pub const NAV_RIGHT:i32=256;
pub const TABS:[usize;5]=[0,2,3,4,5];
pub const NAV_TOP:i32=10;
pub const NAV_BOTTOM:i32=38;
pub const SETTINGS_LEFT:i32=352;
pub const CONTROL_LEFT:i32=312;
pub const BLUETOOTH_LEFT:i32=272;
pub fn centered_left(left:i32,right:i32,width:i32)->i32 {left+(right-left-width)/2}
pub fn content_left(window_left:i32,window_width:i32,layer_width:i32,dock:Option<(i32,i32)>)->i32 {
    if let Some((left,right))=dock {centered_left(left,right,layer_width)-window_left}
    else{(window_width-layer_width)/2}
}
pub fn tab_slot(tab:usize)->Option<usize> {TABS.iter().position(|id|*id==if tab==1 {0}else{tab})}
pub fn utility_hit(x:i32,y:i32,left:i32)->bool {(left..left+32).contains(&x) && (10..38).contains(&y)}

#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct Rect {pub x:i32,pub y:i32,pub w:i32,pub h:i32}
impl Rect {pub fn contains(self,x:i32,y:i32)->bool {(self.x..self.x+self.w).contains(&x) && (self.y..self.y+self.h).contains(&y)}}
// Music uses body design coordinates, before BODY_OFFSET is applied.
pub const MUSIC_CARD:Rect=Rect{x:16,y:90,w:368,h:88};
pub const MUSIC_PLAY:Rect=Rect{x:182,y:216,w:36,h:36};
pub const MUSIC_PREVIOUS:Rect=Rect{x:132,y:218,w:32,h:32};
pub const MUSIC_NEXT:Rect=Rect{x:236,y:218,w:32,h:32};
pub const MUSIC_LYRICS:Rect=Rect{x:328,y:220,w:48,h:28};
pub const MUSIC_SEEK:Rect=Rect{x:24,y:182,w:352,h:16};
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum MusicAction {Play,Previous,Next,Lyrics}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum Action {FocusToggle,FocusReset,Water200,Water400,WaterReset,TodoAdd,ShelfAdd,Seek,Settings,Volume}
pub const FOCUS_TOGGLE:Rect=Rect{x:20,y:196,w:112,h:32};
pub const FOCUS_RESET:Rect=Rect{x:144,y:196,w:88,h:32};
pub const WATER_BUTTONS:[Rect;3]=[Rect{x:20,y:196,w:112,h:32},Rect{x:144,y:196,w:112,h:32},Rect{x:268,y:196,w:112,h:32}];
pub fn body_action(tab:usize,x:i32,y:i32)->Option<Action> {
    match tab {
        2=>[(FOCUS_TOGGLE,Action::FocusToggle),(FOCUS_RESET,Action::FocusReset)].into_iter().find_map(|(r,a)|r.contains(x,y).then_some(a)),
        3=>WATER_BUTTONS.into_iter().zip([Action::Water200,Action::Water400,Action::WaterReset]).find_map(|(r,a)|r.contains(x,y).then_some(a)),
        4|5 if add_hit(x,y)=>Some(if tab==4 {Action::ShelfAdd}else{Action::TodoAdd}),
        0|1 if MUSIC_SEEK.contains(x,y)=>Some(Action::Seek),
        6 if ((204..292).contains(&x)||(304..376).contains(&x)) && (90..110).contains(&y)=>Some(Action::Settings),
        6 if ((24..164).contains(&x)&&(164..186).contains(&y)) || ((24..176).contains(&x)&&(192..216).contains(&y))
            || ((20..174).contains(&x)&&(229..248).contains(&y)) || ((196..384).contains(&x)&&(112..242).contains(&y))=>Some(Action::Settings),
        7 if ((24..64).contains(&x)&&(156..196).contains(&y)) || crate::system_ui::VOLUME.contains(x,y)=>Some(Action::Volume),
        7 if (((16..196).contains(&x)||(204..384).contains(&x))&&(88..148).contains(&y)) || ((16..384).contains(&x)&&(204..244).contains(&y))=>Some(Action::Settings),
        8 if ((16..128).contains(&x)&&(88..112).contains(&y)) || ((28..376).contains(&x)&&(126..166).contains(&y)) || crate::system_ui::TRANSPARENCY.contains(x,y)=>Some(Action::Settings),
        9 if (((16..128).contains(&x)||(300..384).contains(&x))&&(88..112).contains(&y)) || ((16..250).contains(&x)&&(242..258).contains(&y))=>Some(Action::Settings),
        10 if (((16..128).contains(&x)||(264..384).contains(&x))&&(88..112).contains(&y))
            || ((16..384).contains(&x)&&((124..166).contains(&y)||(176..218).contains(&y)))=>Some(Action::Settings),
        11 if ((16..128).contains(&x)&&(88..112).contains(&y)) || ((16..384).contains(&x)&&(120..156).contains(&y))
            || ((220..372).contains(&x)&&(168..192).contains(&y)) || crate::spectrum_ui::SENSITIVITY.contains(x,y)=>Some(Action::Settings),
        12 if ((16..128).contains(&x)&&(88..112).contains(&y)) || ((20..380).contains(&x)&&(210..240).contains(&y))=>Some(Action::Settings),
        _=>None,
    }
}
pub fn music_action(x:i32,y:i32)->Option<MusicAction> {
    [(MUSIC_PLAY,MusicAction::Play),(MUSIC_PREVIOUS,MusicAction::Previous),
        (MUSIC_NEXT,MusicAction::Next),(MUSIC_LYRICS,MusicAction::Lyrics)].into_iter()
        .find_map(|(rect,action)|rect.contains(x,y).then_some(action))
}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum Hover {None,Tab(usize),Settings,Control,Bluetooth,Music(MusicAction),Action(Action),ListRow(usize),ListEdit(usize),ListRemove(usize)}
pub fn hover(tab:usize,x:i32,y:i32)->Hover {
    if utility_hit(x,y,SETTINGS_LEFT) {return Hover::Settings;}
    if utility_hit(x,y,CONTROL_LEFT) {return Hover::Control;}
    if utility_hit(x,y,BLUETOOTH_LEFT) {return Hover::Bluetooth;}
    if (NAV_TOP..NAV_BOTTOM).contains(&y) && (16..NAV_RIGHT).contains(&x) {return Hover::Tab(((x-16)/TAB_STEP).min(4) as usize);}
    if matches!(tab,0|1) {if let Some(action)=music_action(x,y+BODY_OFFSET) {return Hover::Music(action);}}
    let y=y+BODY_OFFSET;
    if let Some(action)=body_action(tab,x,y) {return Hover::Action(action);}
    if matches!(tab,4|5) {if let Some(row)=list_row(x,y) {return if x>=REMOVE_LEFT {Hover::ListRemove(row)}else if tab==5 && x>=EDIT_LEFT {Hover::ListEdit(row)}else{Hover::ListRow(row)};}}
    Hover::None
}
// Body coordinates are translated upward by BODY_OFFSET when drawing, and
// pointer coordinates are translated back before body hit testing.
pub const LIST_TOP:i32=88;
pub const LIST_STEP:i32=30;
pub const LIST_ROW_H:i32=28;
pub const LIST_LEFT:i32=16;
pub const LIST_RIGHT:i32=PANEL_W-16;
pub const REMOVE_LEFT:i32=356;
pub const EDIT_LEFT:i32=314;
pub const ADD_TOP:i32=216;
pub const ADD_H:i32=28;

pub fn list_row(x:i32,y:i32)->Option<usize> {
    if !(LIST_LEFT..LIST_RIGHT).contains(&x) || y<LIST_TOP {return None;}
    let offset=y-LIST_TOP;
    if offset/LIST_STEP<4 && offset%LIST_STEP<LIST_ROW_H {Some((offset/LIST_STEP) as usize)}else{None}
}
pub fn add_hit(x:i32,y:i32)->bool {(LIST_LEFT..LIST_RIGHT).contains(&x) && (ADD_TOP..ADD_TOP+ADD_H).contains(&y)}

#[cfg(test)] mod tests {
    use super::*;
    #[test] fn bluetooth_has_its_own_non_overlapping_navigation_target() {
        for i in 0..TABS.len() {assert_eq!(hover(TABS[i],TAB_START+i as i32*TAB_STEP+TAB_PILL_W/2,24),Hover::Tab(i));}
        assert_eq!(hover(0,BLUETOOTH_LEFT+16,24),Hover::Bluetooth);
        assert_eq!(hover(0,CONTROL_LEFT+16,24),Hover::Control);assert_eq!(hover(0,SETTINGS_LEFT+16,24),Hover::Settings);
    }
    #[test] fn docked_content_is_pixel_locked_across_odd_even_window_sizes() {
        for (left,right) in [(0,1920),(-1920,0),(1920,4480)] {
            for layer in [216,495,600] {
                let expected=centered_left(left,right,layer);
                for width in 200..651 {
                    let window=centered_left(left,right,width);
                    assert_eq!(window+content_left(window,width,layer,Some((left,right))),expected);
                }
            }
        }
        assert_eq!(content_left(123,401,600,None),-99);
    }
    #[test] fn visual_action_hover_matches_click_rectangles_and_keeps_gaps_inactive() {
        for (tab,x,y,expected) in [(2,76,210,Action::FocusToggle),(2,188,210,Action::FocusReset),
            (3,76,210,Action::Water200),(3,200,210,Action::Water400),(3,324,210,Action::WaterReset)] {
            assert_eq!(body_action(tab,x,y),Some(expected));assert_eq!(hover(tab,x,y-BODY_OFFSET),Hover::Action(expected));
        }
        assert_eq!(hover(2,138,210-BODY_OFFSET),Hover::None);
        assert_eq!(hover(5,334,102-BODY_OFFSET),Hover::ListEdit(0));
        assert_eq!(hover(5,370,102-BODY_OFFSET),Hover::ListRemove(0));
        assert_eq!(hover(5,100,102-BODY_OFFSET),Hover::ListRow(0));
        assert_eq!(hover(5,100,117-BODY_OFFSET),Hover::None);
        assert_eq!(hover(5,40,230-BODY_OFFSET),Hover::Action(Action::TodoAdd));
        assert_eq!(hover(4,40,230-BODY_OFFSET),Hover::Action(Action::ShelfAdd));
    }
}
