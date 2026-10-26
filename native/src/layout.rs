// Logical pixels, shared by panel rendering, spring targets and pointer mapping.
pub const PANEL_W:i32=400;
pub const PANEL_H:i32=220;
pub const BODY_OFFSET:i32=40;
pub const TAB_START:i32=18;
pub const TAB_STEP:i32=56;
pub const TAB_PILL_W:i32=52;
pub const TABS:[usize;5]=[0,2,3,4,5];
pub const NAV_TOP:i32=10;
pub const NAV_BOTTOM:i32=38;
pub const SETTINGS_LEFT:i32=352;
pub const CONTROL_LEFT:i32=308;
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
pub fn music_action(x:i32,y:i32)->Option<MusicAction> {
    [(MUSIC_PLAY,MusicAction::Play),(MUSIC_PREVIOUS,MusicAction::Previous),
        (MUSIC_NEXT,MusicAction::Next),(MUSIC_LYRICS,MusicAction::Lyrics)].into_iter()
        .find_map(|(rect,action)|rect.contains(x,y).then_some(action))
}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum Hover {None,Tab(usize),Settings,Control,Music(MusicAction)}
pub fn hover(tab:usize,x:i32,y:i32)->Hover {
    if utility_hit(x,y,SETTINGS_LEFT) {return Hover::Settings;}
    if utility_hit(x,y,CONTROL_LEFT) {return Hover::Control;}
    if (NAV_TOP..NAV_BOTTOM).contains(&y) && (16..296).contains(&x) {return Hover::Tab(((x-16)/TAB_STEP).min(4) as usize);}
    if matches!(tab,0|1) {if let Some(action)=music_action(x,y+BODY_OFFSET) {return Hover::Music(action);}}
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
