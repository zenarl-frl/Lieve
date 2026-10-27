use super::*;
#[derive(Clone,Copy)] pub enum Button {Primary,Secondary,Quiet}
impl App {
    pub(super) unsafe fn action_button(&self,dc:HDC,rect:layout::Rect,label:&str,style:Button,action:layout::Action) {
        let hover=self.hover==layout::Hover::Action(action);
        let (fill,border,text)=match style {
            Button::Primary=>(if hover {rgb(255,255,255)}else{theme::TEXT},theme::TEXT,theme::BACKGROUND),
            Button::Secondary=>(if hover {rgb(40,40,54)}else{theme::LIFT},if hover {rgb(73,73,91)}else{theme::BORDER},theme::TEXT),
            Button::Quiet=>(if hover {theme::LIFT}else{theme::CARD},theme::BORDER,if hover {theme::TEXT}else{theme::MUTED}),
        };
        self.framed_tile(dc,rect,12,fill,border);
        self.fitted_text(dc,label,rect,self.small_font,text,DT_CENTER);
    }
    pub(super) unsafe fn toggle_switch(&self,dc:HDC,rect:layout::Rect,on:bool,background:u32) {
        let track=if on {theme::control_accent(self.color)}else{rgb(50,50,65)};
        self.smooth_tile(dc,rect,rect.h,track,background);
        let size=rect.h-4;let x=if on {rect.x+rect.w-size-2}else{rect.x+2};
        self.smooth_tile(dc,layout::Rect{x,y:rect.y+2,w:size,h:size},size,theme::TEXT,track);
    }
    pub(super) unsafe fn empty_state(&self,dc:HDC,glyph:icons::Glyph,title:&str,detail:&str) {
        self.framed_tile(dc,layout::Rect{x:16,y:88,w:368,h:116},20,theme::CARD,theme::BORDER);
        self.smooth_tile(dc,layout::Rect{x:181,y:100,w:38,h:38},16,theme::LIFT,theme::CARD);
        self.icon(dc,188,107,24,glyph,theme::MUTED,theme::LIFT,None);
        self.fitted_text(dc,title,layout::Rect{x:28,y:144,w:344,h:24},self.font,theme::TEXT,DT_CENTER);
        self.fitted_text(dc,detail,layout::Rect{x:28,y:174,w:344,h:16},self.caption_font,theme::MUTED,DT_CENTER);
    }
    pub(super) unsafe fn list_add_button(&self,dc:HDC,label:&str,count:&str,action:layout::Action,enabled:bool) {
        let hover=enabled && self.hover==layout::Hover::Action(action);
        self.framed_tile(dc,layout::Rect{x:layout::LIST_LEFT,y:layout::ADD_TOP,w:layout::LIST_RIGHT-layout::LIST_LEFT,h:layout::ADD_H},12,
            if hover {rgb(38,32,45)}else{theme::CARD},if hover {rgb(74,59,79)}else{theme::BORDER});
        self.fitted_text(dc,label,layout::Rect{x:28,y:layout::ADD_TOP,w:156,h:layout::ADD_H},self.small_font,if enabled {theme::TEXT}else{theme::MUTED},DT_LEFT);
        self.fitted_text(dc,count,layout::Rect{x:184,y:layout::ADD_TOP,w:188,h:layout::ADD_H},self.caption_font,theme::MUTED,DT_RIGHT);
    }
    pub(super) unsafe fn list_row_style(&self,dc:HDC,row:usize,y:i32)->u32 {
        let hover=matches!(self.hover,layout::Hover::ListRow(i)|layout::Hover::ListEdit(i)|layout::Hover::ListRemove(i) if i==row);
        let bg=if hover {theme::LIFT}else{theme::CARD};
        self.framed_tile(dc,layout::Rect{x:layout::LIST_LEFT,y,w:layout::LIST_RIGHT-layout::LIST_LEFT,h:layout::LIST_ROW_H},12,bg,theme::BORDER);bg
    }
    pub(super) unsafe fn list_icon(&self,dc:HDC,row:usize,y:i32,edit:bool,background:u32) {
        let hover=self.hover==if edit {layout::Hover::ListEdit(row)}else{layout::Hover::ListRemove(row)};
        let x=if edit {327}else{362};
        let fill=if hover {if edit {rgb(43,43,58)}else{rgb(57,27,37)}}else{background};
        if hover {self.smooth_tile(dc,layout::Rect{x:x-5,y:y+3,w:26,h:22},8,fill,background);}
        self.icon(dc,x,y+6,16,if edit {icons::Glyph::Edit}else{icons::Glyph::Close},
            if hover {if edit {theme::TEXT}else{rgb(255,146,160)}}else{theme::MUTED},fill,None);
    }
}
