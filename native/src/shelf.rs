use std::{fs, path::PathBuf};
use super::{App,wide};
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
        if (255..286).contains(&y) {
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
        } else if (100..244).contains(&y) {
            let index=self.shelf_offset+((y-100)/36) as usize;
            if index<self.shelf.len() {
                if x>=424 {
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
            self.text(dc,"Keep your files nearby",28,132,super::rgb(235,235,243),false);
            self.text(dc,"Drop files onto the notch, or choose Add file.",28,169,muted,true);
            self.text(dc,"Files stay in their original location.",28,193,muted,true);
        }
        for (row,file) in self.shelf.iter().skip(self.shelf_offset).take(4).enumerate() {
            let y=100+row as i32*36;
            self.tile(dc,20,y,432,32,12,super::rgb(23,23,30));
            let name=file.file_name().unwrap_or(file.as_os_str()).to_string_lossy();
            self.text(dc,&super::visual::shorten(&name,48),30,y+2,super::rgb(235,235,243),true);
            // Do not stat paths during paint: removable/network drives may be slow.
            self.text(dc,"Saved reference · click to open",30,y+17,muted,true);
            self.text(dc,"x",435,y+8,muted,true);
        }
        self.tile(dc,20,255,432,30,12,super::rgb(38,27,34));
        self.text(dc,"+ Add file  /  drop files here",110,262,self.color,true);
        self.text(dc,&format!("{} items · scroll for more · x removes reference only",self.shelf.len()),24,298,muted,true);
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
