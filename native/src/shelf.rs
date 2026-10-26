use std::{fs, path::PathBuf};
use super::{App,wide,layout};
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
        let muted=super::rgb(135,135,151);
        if self.shelf.is_empty() {
            self.text(dc,"Keep your files nearby",24,122,super::rgb(235,235,243),false);
            self.text(dc,"Drop files here or use Add file.",24,155,muted,true);
            self.text(dc,"Files stay in their original location.",24,177,muted,true);
        }
        for (row,file) in self.shelf.iter().skip(self.shelf_offset).take(4).enumerate() {
            let y=layout::LIST_TOP+row as i32*layout::LIST_STEP;
            self.tile(dc,layout::LIST_LEFT,y,layout::LIST_RIGHT-layout::LIST_LEFT,layout::LIST_ROW_H,12,super::rgb(23,23,30));
            let name=file.file_name().unwrap_or(file.as_os_str()).to_string_lossy();
            SaveDC(dc); IntersectClipRect(dc,self.px(26),self.px(y),self.px(layout::REMOVE_LEFT-4),self.px(y+layout::LIST_ROW_H));
            self.text(dc,&super::visual::shorten(&name,40),26,y+1,super::rgb(235,235,243),true);
            // Do not stat paths during paint: removable/network drives may be slow.
            self.text(dc,"Click to open · reference",26,y+14,muted,true);
            RestoreDC(dc,-1);
            self.text(dc,"x",370,y+6,muted,true);
        }
        self.tile(dc,layout::LIST_LEFT,layout::ADD_TOP,layout::LIST_RIGHT-layout::LIST_LEFT,layout::ADD_H,12,super::rgb(38,27,34));
        self.fitted_text(dc,"+ Add file",layout::Rect{x:28,y:layout::ADD_TOP,w:240,h:layout::ADD_H},self.small_font,self.color,DT_LEFT);
        self.fitted_text(dc,&format!("{} {}",self.shelf.len(),if self.shelf.len()==1 {"item"}else{"items"}),layout::Rect{x:272,y:layout::ADD_TOP,w:100,h:layout::ADD_H},self.small_font,muted,DT_RIGHT);
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
