use super::*;
use windows_sys::Win32::UI::Controls::{EM_SETLIMITTEXT, EM_SETSEL};
use std::{fs,path::PathBuf};

#[derive(Clone, PartialEq, Eq)]
pub struct Task { pub text:String, pub done:bool }
fn path()->PathBuf { PathBuf::from(std::env::var_os("APPDATA").unwrap_or_default()).join("Lieve").join("tasks.txt") }
pub fn parse(text:&str)->Vec<Task> {
    text.lines().filter_map(|line| {
        let (flag,text)=line.split_once('\t')?;
        if !matches!(flag,"0"|"1") || text.trim().is_empty() {return None;}
        Some(Task{text:text.chars().take(200).collect(),done:flag=="1"})
    }).take(100).collect()
}
pub fn load()->Vec<Task> { parse(&fs::read_to_string(path()).unwrap_or_default()) }
pub fn save(tasks:&[Task])->std::io::Result<()> {
    let file=path(); fs::create_dir_all(file.parent().unwrap())?;
    let text=tasks.iter().map(|t|format!("{}\t{}",u8::from(t.done),t.text.replace(['\r','\n','\t']," "))).collect::<Vec<_>>().join("\n");
    let temp=file.with_extension("tmp"); fs::write(&temp,text)?; fs::rename(temp,file)
}

struct Input {edit:HWND, result:Option<String>}
unsafe extern "system" fn input_proc(hwnd:HWND,msg:u32,wp:WPARAM,lp:LPARAM)->LRESULT {
    if msg==WM_NCCREATE {
        SetWindowLongPtrW(hwnd,GWLP_USERDATA,(*(lp as *const CREATESTRUCTW)).lpCreateParams as isize);
        return DefWindowProcW(hwnd,msg,wp,lp);
    }
    if msg==WM_CLOSE {DestroyWindow(hwnd); return 0;}
    if msg==WM_DESTROY {
        // Wake GetMessage even when a synchronous cross-thread command closes the
        // dialog while the modal loop is waiting for a queued message.
        PostMessageW(GetWindow(hwnd,GW_OWNER),WM_NULL,0,0); return 0;
    }
    if msg==WM_COMMAND {
        let state=GetWindowLongPtrW(hwnd,GWLP_USERDATA) as *mut Input;
        if !state.is_null() {
            match wp&0xffff {
                1=>{
                    let mut buffer=[0u16;201];
                    let len=GetWindowTextW((*state).edit,buffer.as_mut_ptr(),201);
                    let value=String::from_utf16_lossy(&buffer[..len.max(0) as usize]).trim().to_string();
                    if value.is_empty() {return 0;}
                    (*state).result=Some(value); DestroyWindow(hwnd); return 0;
                }
                2=>{DestroyWindow(hwnd); return 0;}
                _=>{}
            }
        }
    }
    DefWindowProcW(hwnd,msg,wp,lp)
}

unsafe fn input(owner:HWND,existing:Option<&str>)->Option<String> {
    let class=wide("LieveTaskInput"); let instance=GetModuleHandleW(null());
    let wc=WNDCLASSW {lpfnWndProc:Some(input_proc),hInstance:instance,lpszClassName:class.as_ptr(),hCursor:LoadCursorW(null_mut(),IDC_ARROW),hbrBackground:(COLOR_WINDOW+1) as HBRUSH,..std::mem::zeroed()};
    RegisterClassW(&wc);
    let mut state=Input{edit:null_mut(),result:None};
    let mut rect=std::mem::zeroed::<RECT>(); GetWindowRect(owner,&mut rect);
    let scale=GetDpiForWindow(owner) as f32/96.; let px=|n:i32|(n as f32*scale) as i32;
    let title=if existing.is_some() {"Lieve — Edit task"}else{"Lieve — Add task"};
    let hwnd=CreateWindowExW(WS_EX_DLGMODALFRAME|WS_EX_CONTROLPARENT,class.as_ptr(),wide(title).as_ptr(),WS_CAPTION|WS_SYSMENU,
        rect.left,rect.top+px(40),px(420),px(160),owner,null_mut(),instance,(&mut state as *mut Input).cast());
    if hwnd.is_null() {return None;}
    let child=|kind:&str,text:&str,style:u32,x,y,w,h,id| {
        let control=CreateWindowExW(0,wide(kind).as_ptr(),wide(text).as_ptr(),WS_CHILD|WS_VISIBLE|style,px(x),px(y),px(w),px(h),hwnd,id as HMENU,instance,null());
        SendMessageW(control,WM_SETFONT,GetStockObject(DEFAULT_GUI_FONT) as usize,1); control
    };
    child("STATIC","What would you like to do?",0,16,12,380,22,0usize);
    state.edit=child("EDIT",existing.unwrap_or_default(),WS_BORDER|WS_TABSTOP|ES_AUTOHSCROLL as u32,16,38,380,26,0usize);
    SendMessageW(state.edit,EM_SETLIMITTEXT,200,0);
    child("BUTTON",if existing.is_some() {"Save"}else{"Add"},WS_TABSTOP|BS_DEFPUSHBUTTON as u32,224,78,80,28,1usize);
    child("BUTTON","Cancel",WS_TABSTOP,316,78,80,28,2usize);
    EnableWindow(owner,0); ShowWindow(hwnd,SW_SHOW); SetFocus(state.edit);
    SendMessageW(state.edit,EM_SETSEL,0,-1);
    let mut msg=std::mem::zeroed::<MSG>();
    while IsWindow(hwnd)!=0 {
        let status=GetMessageW(&mut msg,null_mut(),0,0);
        if status<=0 {if status==0 {PostQuitMessage(msg.wParam as i32);} break;}
        if IsDialogMessageW(hwnd,&msg)==0 {TranslateMessage(&msg); DispatchMessageW(&msg);}
    }
    if IsWindow(hwnd)!=0 {DestroyWindow(hwnd);}
    EnableWindow(owner,1); SetForegroundWindow(owner);
    state.result
}

impl App {
    pub unsafe fn todo_click(&mut self,hwnd:HWND,x:i32,y:i32) {
        let previous=self.tasks.clone();
        let previous_offset=self.task_offset;
        if layout::add_hit(x,y) && self.tasks.len()<100 {
            self.menu_open=true;
            if let Some(text)=input(hwnd,None) {self.tasks.insert(0,Task{text,done:false}); self.task_offset=0;}
            self.menu_open=false;
        } else if let Some(row)=layout::list_row(x,y) {
            let index=self.task_offset+row;
            if index<self.tasks.len() {
                if x>=layout::REMOVE_LEFT {self.tasks.remove(index); self.task_offset=self.task_offset.min(self.tasks.len().saturating_sub(4));}
                else if x>=layout::EDIT_LEFT {
                    self.menu_open=true;
                    if let Some(text)=input(hwnd,Some(&self.tasks[index].text)) {self.tasks[index].text=text;}
                    self.menu_open=false;
                }
                else {self.tasks[index].done=!self.tasks[index].done;}
            }
        }
        if self.tasks!=previous {
            if let Err(error)=save(&self.tasks) {
                self.tasks=previous; self.task_offset=previous_offset;
                MessageBoxW(hwnd,wide(&format!("Cannot save tasks. Your change was not applied: {error}")).as_ptr(),wide("Lieve").as_ptr(),MB_OK|MB_ICONWARNING);
            }
        }
        InvalidateRect(hwnd,null(),0);
    }
    pub unsafe fn todo_view(&self,dc:HDC) {
        let muted=rgb(135,135,151);
        if self.tasks.is_empty() {self.text(dc,"A quieter place for your tasks",24,132,rgb(235,235,243),false);}
        for (row,task) in self.tasks.iter().skip(self.task_offset).take(4).enumerate() {
            let y=layout::LIST_TOP+row as i32*layout::LIST_STEP;
            self.tile(dc,layout::LIST_LEFT,y,layout::LIST_RIGHT-layout::LIST_LEFT,layout::LIST_ROW_H,12,rgb(23,23,30));
            self.tile(dc,26,y+7,14,14,6,if task.done {self.color}else{rgb(64,64,76)});
            SaveDC(dc);
            IntersectClipRect(dc,self.px(48),self.px(y),self.px(layout::EDIT_LEFT-4),self.px(y+layout::LIST_ROW_H));
            self.text(dc,&visual::shorten(&task.text,31),48,y+6,if task.done {muted}else{rgb(235,235,243)},true);
            RestoreDC(dc,-1);
            self.text(dc,"edit",322,y+6,muted,true);
            self.text(dc,"x",370,y+6,muted,true);
        }
        self.tile(dc,layout::LIST_LEFT,layout::ADD_TOP,layout::LIST_RIGHT-layout::LIST_LEFT,layout::ADD_H,12,rgb(38,27,34));
        self.fitted_text(dc,"+ Add task",layout::Rect{x:28,y:layout::ADD_TOP,w:240,h:layout::ADD_H},self.small_font,self.color,DT_LEFT);
        self.fitted_text(dc,&format!("{} / {} done",self.tasks.iter().filter(|t|t.done).count(),self.tasks.len()),layout::Rect{x:272,y:layout::ADD_TOP,w:100,h:layout::ADD_H},self.small_font,muted,DT_RIGHT);
    }
}

#[cfg(test)] mod tests {
    use super::*;
    #[test] fn reads_task_states() {let t=parse("1\tDone\n0\tPending");assert_eq!(t.len(),2);assert!(t[0].done);assert!(!t[1].done);}
    #[test] fn ignores_invalid_and_empty() {assert!(parse("invalid\n2\tbad\n0\t ").is_empty());}
    #[test] fn limits_text_and_count() {let s=format!("0\t{}\n","x".repeat(300)).repeat(110);let t=parse(&s);assert_eq!(t.len(),100);assert_eq!(t[0].text.len(),200);}
}
