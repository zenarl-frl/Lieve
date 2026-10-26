use std::{fs, path::PathBuf};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Motion { System, Full, Reduced }
impl Motion {
    pub fn enabled(self,windows_enabled:bool)->bool {
        match self {Self::System=>windows_enabled,Self::Full=>true,Self::Reduced=>false}
    }
    fn label(self)->&'static str {match self {Self::System=>"system",Self::Full=>"full",Self::Reduced=>"reduced"}}
}

#[derive(Debug, PartialEq)]
pub struct Settings {
    pub motion: Motion,
    pub mascot: bool,
    pub devil_grin: bool,
    pub transparency:u32,
    pub frosted:bool,
    pub compact_lyrics: bool,
    pub glow: u32,
    pub color: u32,
    pub floating: bool,
    pub minimal: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self { motion:Motion::System, mascot:true, devil_grin:false, transparency:25,frosted:true,compact_lyrics:true, glow:50, color: 226 | 54 << 8 | 54 << 16, floating: false, minimal: false }
    }
}

fn folder() -> PathBuf {
    PathBuf::from(std::env::var_os("APPDATA").unwrap_or_default()).join("Lieve")
}

impl Settings {
    pub fn parse(text: &str) -> Self {
        let mut result = Self::default();
        for line in text.lines() {
            if let Some((key, value)) = line.split_once('=') {
                match key {
                    "motion" => result.motion=match value {"full"=>Motion::Full,"reduced"=>Motion::Reduced,_=>Motion::System},
                    "color" => if let Ok(color) = value.parse::<u32>() {
                        if color <= 0xFFFFFF { result.color = color; }
                    },
                    "floating" => result.floating = value == "true",
                    "minimal" => result.minimal = value == "true",
                    "mascot" => result.mascot = value != "false",
                    "devil_grin" => result.devil_grin = value == "true",
                    "frosted"=>result.frosted=value!="false",
                    "transparency"=>if let Ok(v)=value.parse::<u32>() {result.transparency=v.min(100);},
                    "compact_lyrics" => result.compact_lyrics = value != "false",
                    "glow" => if let Ok(glow)=value.parse::<u32>() { result.glow=glow.min(100); },
                    _ => {}
                }
            }
        }
        result
    }

    fn encode(&self) -> String {
        format!("color={}\nfloating={}\nminimal={}\nmascot={}\ncompact_lyrics={}\nglow={}\nmotion={}\ndevil_grin={}\ntransparency={}\nfrosted={}\n", self.color, self.floating, self.minimal,self.mascot,self.compact_lyrics,self.glow,self.motion.label(),self.devil_grin,self.transparency,self.frosted)
    }

    pub fn load() -> Self {
        if let Ok(text) = fs::read_to_string(folder().join("settings.txt")) { return Self::parse(&text); }
        let mut settings = Self::default();
        if let Some(color) = fs::read_to_string(folder().join("eye-color.txt")).ok()
            .and_then(|s| s.trim().parse::<u32>().ok()).filter(|c| *c <= 0xFFFFFF) {
            settings.color = color;
        }
        settings
    }

    pub fn save(&self) -> std::io::Result<()> {
        fs::create_dir_all(folder())?;
        let temp = folder().join("settings.tmp");
        fs::write(&temp, self.encode())?;
        fs::rename(temp, folder().join("settings.txt"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn settings_round_trip() {
        let original = Settings { motion:Motion::Full, color: 0x123456, floating: true, minimal: true, mascot:false, devil_grin:true,transparency:63,frosted:false, compact_lyrics:false, glow:77 };
        assert_eq!(Settings::parse(&original.encode()), original);
    }
    #[test]
    fn invalid_color_keeps_default() {
        assert_eq!(Settings::parse("color=4294967295").color, Settings::default().color);
        assert_eq!(Settings::parse("color=nope").color, Settings::default().color);
    }
    #[test] fn motion_override_and_legacy_settings() {
        assert_eq!(Settings::parse("").motion,Motion::System);
        assert_eq!(Settings::parse("motion=invalid").motion,Motion::System);
        assert!(!Motion::System.enabled(false)); assert!(Motion::System.enabled(true));
        assert!(Motion::Full.enabled(false)); assert!(!Motion::Reduced.enabled(true));
        for mode in [Motion::System,Motion::Full,Motion::Reduced] {
            let settings=Settings{motion:mode,..Settings::default()};
            assert_eq!(Settings::parse(&settings.encode()),settings);
        }
    }
    #[test] fn glow_is_bounded() {
        assert_eq!(Settings::parse("glow=500").glow,100);
        assert_eq!(Settings::parse("glow=bad").glow,50);
    }
    #[test] fn transparency_is_bounded_and_old_files_keep_defaults() {
        assert_eq!(Settings::parse("transparency=999").transparency,100);
        assert_eq!(Settings::parse("transparency=oops").transparency,25);
        assert_eq!(Settings::parse("transparency=0\nfrosted=false").transparency,0);
        assert!(!Settings::parse("frosted=false").frosted);
    }
    #[test]
    fn missing_fields_and_unknown_keys_are_safe() {
        assert_eq!(Settings::parse("unknown=hello\nminimal=true"), Settings { minimal: true, ..Settings::default() });
    }
}
