use std::{fs, path::PathBuf};
use super::{App,wide,layout,theme,icons};
use windows_sys::Win32::{Foundation::*,UI::{WindowsAndMessaging::*,Controls::Dialogs::*,Shell::*},Graphics::Gdi::*};

const LIMIT: usize = 100;
fn path() -> PathBuf {
    PathBuf::from(std::env::var_os("APPDATA").unwrap_or_default()).join("Lieve").join("shelf.txt")
}

pub fn add(items: &mut Vec<PathBuf>, file: PathBuf) -> bool {
    // Keep stale references visible after restart, but accept only existing new items.
    if !file.is_absolute() || !file.exists() || items.contains(&file) || items.len() >= LIMIT {
        return false;
    }
    items.insert(0,file);
    true
}

pub fn load() -> Vec<PathBuf> {
    let text=fs::read_to_string(path()).unwrap_or_default();
    let mut items=Vec::new();
    for line in text.lines().take(LIMIT) {
        let file=PathBuf::from(line);
        if file.is_absolute() && !items.contains(&file) { items.push(file); }
    }
    items
}

pub fn save(items: &[PathBuf]) -> std::io::Result<()> {
    let path=path();
    fs::create_dir_all(path.parent().unwrap())?;
    let text=items.iter().take(LIMIT).map(|p|p.to_string_lossy()).collect::<Vec<_>>().join("\n");
    let temp=path.with_extension("tmp");
    fs::write(&temp,text)?;
    fs::rename(temp,path)
}

impl App {
    pub unsafe fn save_shelf(&self, hwnd:HWND) {
        if let Err(error)=save(&self.shelf) {
            MessageBoxW(hwnd,wide(&format!("Could not save shelf references: {error}")).as_ptr(),wide("Lieve").as_ptr(),MB_OK|MB_ICONWARNING);
        }
    }

    pub unsafe fn shelf_click(&mut self, hwnd:HWND, x:i32, y:i32) {
        if layout::add_hit(x,y) {
            self.menu_open=true;
            let mut buffer=vec![0u16;32768];
            let mut dialog=std::mem::zeroed::<OPENFILENAMEW>();
            dialog.lStructSize=std::mem::size_of::<OPENFILENAMEW>() as u32;
            dialog.hwndOwner=hwnd;
            dialog.lpstrFile=buffer.as_mut_ptr(); dialog.nMaxFile=buffer.len() as u32;
            let filter=wide("All files\0*.*\0");
            let title=wide("Add file to Lieve shelf (reference only)");
            dialog.lpstrFilter=filter.as_ptr(); dialog.lpstrTitle=title.as_ptr();
            dialog.Flags=OFN_FILEMUSTEXIST|OFN_PATHMUSTEXIST|OFN_NOCHANGEDIR|OFN_EXPLORER;
            if GetOpenFileNameW(&mut dialog)!=0 {
                use std::os::windows::ffi::OsStringExt;
                let len=buffer.iter().position(|c|*c==0).unwrap_or(buffer.len());
                add(&mut self.shelf,PathBuf::from(std::ffi::OsString::from_wide(&buffer[..len])));
                self.shelf_offset=0; self.save_shelf(hwnd);
            }
            self.menu_open=false;
        } else if let Some(row)=layout::list_row(x,y) {
            let index=self.shelf_offset+row;
            if index<self.shelf.len() {
                if x>=layout::REMOVE_LEFT {
                    self.shelf.remove(index);
                    self.shelf_offset=self.shelf_offset.min(self.shelf.len().saturating_sub(4));
                    self.save_shelf(hwnd);
                } else {
                    let file=&self.shelf[index];
                    if file.exists() {
                        use std::os::windows::ffi::OsStrExt;
                        let path:Vec<u16>=file.as_os_str().encode_wide().chain(Some(0)).collect();
                        let result=ShellExecuteW(hwnd,wide("open").as_ptr(),path.as_ptr(),std::ptr::null(),std::ptr::null(),SW_SHOWNORMAL);
                        if result as isize<=32 {
                            MessageBoxW(hwnd,wide("Windows could not open this item.").as_ptr(),wide("Lieve").as_ptr(),MB_OK|MB_ICONWARNING);
                        }
                    } else {
                        MessageBoxW(hwnd,wide("This file was moved or deleted. Remove its reference and add it again.").as_ptr(),wide("Lieve").as_ptr(),MB_OK|MB_ICONWARNING);
                    }
                }
            }
        }
        InvalidateRect(hwnd,std::ptr::null(),0);
    }

    pub unsafe fn shelf_view(&self, dc:HDC) {
        let muted=theme::MUTED;
        if self.shelf.is_empty() {
            self.empty_state(dc,icons::Glyph::File,"Keep your files close","Drop a file · Originals stay where they are.");
        }
        for (row,file) in self.shelf.iter().skip(self.shelf_offset).take(4).enumerate() {
            let y=layout::LIST_TOP+row as i32*layout::LIST_STEP;
            let bg=self.list_row_style(dc,row,y);self.icon(dc,26,y+6,16,icons::Glyph::File,muted,bg,None);
            let name=file.file_name().unwrap_or(file.as_os_str()).to_string_lossy();
            self.fitted_text(dc,&name,layout::Rect{x:50,y:y+2,w:layout::REMOVE_LEFT-58,h:14},self.small_font,theme::TEXT,DT_LEFT);
            // Do not stat paths during paint: removable/network drives may be slow.
            let parent=file.parent().map(|path|path.to_string_lossy()).unwrap_or_else(||"Local file reference".into());
            self.fitted_text(dc,&parent,layout::Rect{x:50,y:y+16,w:layout::REMOVE_LEFT-58,h:11},self.caption_font,muted,DT_LEFT);
            self.list_icon(dc,row,y,false,bg);
        }
        let count=if self.shelf.len()>4 {format!("{}–{} / {} files",self.shelf_offset+1,(self.shelf_offset+4).min(self.shelf.len()),self.shelf.len())}else{format!("{} {}",self.shelf.len(),if self.shelf.len()==1 {"item"}else{"items"})};
        self.list_add_button(dc,if self.shelf.len()<100 {"+ Add file"}else{"File limit reached"},&count,layout::Action::ShelfAdd,self.shelf.len()<100);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn reject_relative_and_missing_files() {
        let mut items=Vec::new();
        assert!(!add(&mut items,PathBuf::from("relative.txt")));
        assert!(!add(&mut items,std::env::temp_dir().join("lieve-nonexistent-6a058d35")));
    }
    #[test] fn references_do_not_copy_or_duplicate_files() {
        let file=std::env::current_exe().unwrap();
        let mut items=Vec::new();
        assert!(add(&mut items,file.clone()));
        assert!(!add(&mut items,file.clone()));
        items.clear();
        assert!(file.exists());
    }
}
