use std::sync::mpsc::{self, Receiver, Sender};
use windows::{core::HSTRING, Data::Json::JsonObject, Foundation::Uri, Web::Http::HttpClient};

#[derive(Debug, PartialEq)]
pub struct Line { pub at: u32, pub text: String }

pub struct Request { pub key: String, pub title: String, pub artist: String, pub duration: u32 }
pub struct Result { pub key: String, pub lines: Vec<Line> }

fn encode(value: &str) -> String {
    value.bytes().map(|b| {
        if b.is_ascii_alphanumeric() || b"-._~".contains(&b) { (b as char).to_string() }
        else { format!("%{b:02X}") }
    }).collect()
}

pub fn parse(text: &str) -> Vec<Line> {
    let mut lines = Vec::new();
    for raw in text.lines() {
        let mut rest = raw;
        let mut times = Vec::new();
        while let Some(bracket) = rest.strip_prefix('[') {
            let Some((stamp, tail)) = bracket.split_once(']') else { break; };
            rest = tail;
            let Some((minute, second)) = stamp.split_once(':') else { continue; };
            if let (Ok(m), Ok(s)) = (minute.parse::<u32>(), second.parse::<f64>()) {
                if s.is_finite() && (0.0..60.0).contains(&s) && m < 1440 {
                    times.push(m*1000*60+(s*1000.) as u32);
                }
            }
        }
        for at in times { lines.push(Line { at, text: rest.trim().to_string() }); }
    }
    lines.sort_by_key(|l| l.at);
    lines
}

pub fn start() -> (Sender<Request>, Receiver<Result>) {
    let (tx, requests) = mpsc::channel::<Request>();
    let (results, rx) = mpsc::channel();
    std::thread::spawn(move || {
        unsafe { super::initialize_runtime(); }
        while let Ok(mut request) = requests.recv() {
            while let Ok(newer) = requests.try_recv() { request = newer; }
            let lines = (|| -> windows::core::Result<Vec<Line>> {
                let url = format!("https://lrclib.net/api/get?track_name={}&artist_name={}&duration={}",encode(&request.title),encode(&request.artist),request.duration);
                let client = HttpClient::new()?;
                let body = client.GetStringAsync(&Uri::CreateUri(&HSTRING::from(url))?)?.get()?;
                let json = JsonObject::Parse(&body)?;
                let lrc = json.GetNamedString(&HSTRING::from("syncedLyrics"))?;
                Ok(parse(&lrc.to_string()))
            })().unwrap_or_default();
            if results.send(Result { key:request.key, lines }).is_err() { break; }
        }
    });
    (tx, rx)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn timestamps_and_repeated_lines() {
        let lines=parse("[ar:someone]\n[00:12.50][01:00.00]hello\n[00:01.000]first");
        assert_eq!(lines.len(),3); assert_eq!(lines[0].at,1000); assert_eq!(lines[1].at,12500);
        assert_eq!(lines[2].text,"hello");
    }
    #[test] fn invalid_times_are_ignored() { assert!(parse("[00:NaN]bad\n[00:99]bad\nno time").is_empty()); }
    #[test] fn query_encoding() { assert_eq!(encode("A&B /"),"A%26B%20%2F"); }
    #[test]
    #[ignore = "uses the public LRCLIB service"]
    fn lrclib_live_lookup() {
        let (requests, results)=start();
        requests.send(Request { key:"test".into(), title:"Yesterday".into(), artist:"The Beatles".into(), duration:125 }).unwrap();
        let result=results.recv_timeout(std::time::Duration::from_secs(30)).expect("lyrics service response");
        assert!(!result.lines.is_empty(), "No synced lyrics returned for test song");
    }
}
