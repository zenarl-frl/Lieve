use std::{fs, path::PathBuf};

fn path() -> PathBuf {
    PathBuf::from(std::env::var_os("APPDATA").unwrap_or_default()).join("Lieve").join("water.txt")
}

pub fn today() -> String {
    let mut time = unsafe { std::mem::zeroed::<windows_sys::Win32::Foundation::SYSTEMTIME>() };
    unsafe { super::GetLocalTime(&mut time); }
    format!("{:04}-{:02}-{:02}",time.wYear,time.wMonth,time.wDay)
}

fn parse(text: &str, day: &str) -> u32 {
    let mut lines=text.lines();
    if lines.next() != Some(day) { return 0; }
    lines.next().and_then(|s| s.parse().ok()).unwrap_or(0)
}

pub fn load(day: &str) -> u32 {
    fs::read_to_string(path()).map(|s| parse(&s,day)).unwrap_or(0)
}

pub fn save(day: &str, ml: u32) -> std::io::Result<()> {
    let path=path();
    fs::create_dir_all(path.parent().unwrap())?;
    let temp=path.with_extension("tmp");
    fs::write(&temp, format!("{day}\n{ml}\n"))?;
    fs::rename(temp,path)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn same_day_retains_intake() { assert_eq!(parse("2026-10-25\n600\n","2026-10-25"),600); }
    #[test] fn new_day_resets_intake() { assert_eq!(parse("2026-10-24\n600\n","2026-10-25"),0); }
    #[test] fn corrupt_values_are_safe() { assert_eq!(parse("2026-10-25\ninvalid","2026-10-25"),0); }
}
