use std::{io,path::Path,ptr::{null,null_mut}};
use windows_sys::Win32::{Foundation::{ERROR_SUCCESS,ERROR_FILE_NOT_FOUND},System::Registry::*};
const RUN:&str="Software\\Microsoft\\Windows\\CurrentVersion\\Run";
const NAME:&str="Lieve";
struct Key(HKEY);
impl Drop for Key {fn drop(&mut self) {unsafe {RegCloseKey(self.0);}}}
fn result(code:u32)->io::Result<()> {if code==ERROR_SUCCESS {Ok(())}else{Err(io::Error::from_raw_os_error(code as i32))}}
fn command(executable:&Path)->io::Result<Vec<u16>> {
    use std::os::windows::ffi::OsStrExt;
    let path:Vec<u16>=executable.as_os_str().encode_wide().collect();
    if path.iter().any(|v|*v==0 || *v==b'"' as u16) {return Err(io::Error::new(io::ErrorKind::InvalidInput,"Invalid executable path"));}
    Ok(std::iter::once(b'"' as u16).chain(path).chain([b'"' as u16,0]).collect())
}
fn read_at(path:&str)->io::Result<Option<Vec<u16>>> {
    let mut bytes=0;let subkey=crate::wide(path);let name=crate::wide(NAME);
    unsafe {
        let code=RegGetValueW(HKEY_CURRENT_USER,subkey.as_ptr(),name.as_ptr(),RRF_RT_REG_SZ,null_mut(),null_mut(),&mut bytes);
        if code==ERROR_FILE_NOT_FOUND {return Ok(None);}result(code)?;
        let mut buffer=vec![0u16;(bytes as usize).div_ceil(2)];
        result(RegGetValueW(HKEY_CURRENT_USER,subkey.as_ptr(),name.as_ptr(),RRF_RT_REG_SZ,null_mut(),buffer.as_mut_ptr().cast(),&mut bytes))?;
        buffer.truncate((bytes as usize).div_ceil(2));Ok(Some(buffer))
    }
}
fn set_at(path:&str,value:Option<&[u16]>)->io::Result<()> {
    let subkey=crate::wide(path);let name=crate::wide(NAME);let mut handle=null_mut();
    unsafe {
        if let Some(value)=value {
            result(RegCreateKeyExW(HKEY_CURRENT_USER,subkey.as_ptr(),0,null(),0,KEY_SET_VALUE,null(),&mut handle,null_mut()))?;
            let key=Key(handle);result(RegSetValueExW(key.0,name.as_ptr(),0,REG_SZ,value.as_ptr().cast(),(value.len()*2) as u32))
        }else {
            let code=RegOpenKeyExW(HKEY_CURRENT_USER,subkey.as_ptr(),0,KEY_SET_VALUE,&mut handle);
            if code==ERROR_FILE_NOT_FOUND {return Ok(());}result(code)?;let key=Key(handle);
            let code=RegDeleteValueW(key.0,name.as_ptr());if code==ERROR_FILE_NOT_FOUND {Ok(())}else{result(code)}
        }
    }
}
pub fn enabled()->io::Result<bool> {
    let expected=command(&std::env::current_exe()?)?;
    Ok(read_at(RUN)?.is_some_and(|value|String::from_utf16_lossy(&value).eq_ignore_ascii_case(&String::from_utf16_lossy(&expected))))
}
pub fn set(enabled:bool)->io::Result<()> {
    if enabled {set_at(RUN,Some(&command(&std::env::current_exe()?)?))}else{set_at(RUN,None)}
}
#[cfg(test)] mod tests {
    use super::*;
    #[test] fn executable_path_is_quoted_for_windows_startup() {
        let value=command(Path::new("C:\\Program Files\\Lieve\\lieve.exe")).unwrap();
        assert_eq!(String::from_utf16(&value[..value.len()-1]).unwrap(),"\"C:\\Program Files\\Lieve\\lieve.exe\"");
        assert!(command(Path::new("bad\"path")).is_err());
    }
    #[test] #[ignore="writes and removes a private HKCU test key, never the Windows Run key"]
    fn private_registry_round_trip() {
        let path=format!("Software\\Lieve\\Tests\\Startup-{}-{}",std::process::id(),std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos());
        struct Cleanup(String);impl Drop for Cleanup {fn drop(&mut self) {unsafe {RegDeleteTreeW(HKEY_CURRENT_USER,crate::wide(&self.0).as_ptr());}}}
        let _cleanup=Cleanup(path.clone());let expected=command(Path::new("C:\\A folder\\Lieve\\lieve.exe")).unwrap();
        assert!(read_at(&path).unwrap().is_none());set_at(&path,Some(&expected)).unwrap();assert_eq!(read_at(&path).unwrap(),Some(expected));
        set_at(&path,None).unwrap();assert!(read_at(&path).unwrap().is_none());set_at(&path,None).unwrap();
    }
}
