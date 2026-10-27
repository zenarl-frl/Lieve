//! Local-only spectrum of the default Windows output, never the microphone.
use std::{sync::{Arc,Mutex,atomic::{AtomicBool,Ordering}},time::{Duration,Instant}};
use windows::{core::{GUID,Result},Win32::{Media::Audio::*,System::Com::*}};

pub const BANDS:usize=5;
const N:usize=2048;
const EDGES:[f32;BANDS+1]=[40.,160.,630.,2500.,8000.,16000.];
#[derive(Clone,Copy)]
pub struct Reading {pub levels:[f32;BANDS],pub at:Instant}
impl Reading {
    pub fn sensitivity(mut self,percent:u32)->Self {
        let gain=20.*(percent.clamp(50,200) as f32/100.).log10()/60.;
        self.levels=self.levels.map(|v|if v>0. {(v+gain).clamp(0.,1.)}else{0.});self
    }
}
struct Shared {running:AtomicBool,enabled:AtomicBool,reading:Mutex<Reading>}
pub struct Capture {shared:Arc<Shared>}
impl Capture {
    pub fn start()->Self {
        let shared=Arc::new(Shared{running:AtomicBool::new(true),enabled:AtomicBool::new(false),reading:Mutex::new(Reading{levels:[0.;BANDS],at:Instant::now()})});
        let worker=shared.clone();std::thread::spawn(move || {
            unsafe {if CoInitializeEx(None,COINIT_MULTITHREADED).is_err() {return;}}
            let mut stream:Option<Stream>=None;let mut retry=Instant::now();let mut check=Instant::now();
            while worker.running.load(Ordering::Relaxed) {
                if !worker.enabled.load(Ordering::Relaxed) {
                    stream=None;publish(&worker,[0.;BANDS]);retry=Instant::now();
                    std::thread::sleep(Duration::from_millis(80));continue;
                }
                if stream.is_none() && Instant::now()>=retry {
                    stream=unsafe {Stream::open().ok()};retry=Instant::now()+Duration::from_secs(2);check=Instant::now();
                }
                if check.elapsed()>=Duration::from_secs(2) {
                    if let Some(current)=&stream {if unsafe {default_device().map(|(_,id)|id!=current.id).unwrap_or(true)} {stream=None;retry=Instant::now();}}
                    check=Instant::now();
                }
                if let Some(current)=&mut stream {
                    if unsafe {current.poll(&worker)}.is_err() {stream=None;publish(&worker,[0.;BANDS]);retry=Instant::now()+Duration::from_secs(2);}
                }
                std::thread::sleep(Duration::from_millis(16));
            }
            drop(stream);unsafe {CoUninitialize();}
        });Self{shared}
    }
    pub fn enabled(&self)->bool {self.shared.enabled.load(Ordering::Relaxed)}
    pub fn enable(&self,value:bool)->bool {self.shared.enabled.swap(value,Ordering::Relaxed)!=value}
    pub fn reading(&self)->Reading {*self.shared.reading.lock().unwrap_or_else(|e|e.into_inner())}
}
impl Drop for Capture {fn drop(&mut self) {self.shared.running.store(false,Ordering::Relaxed);}}
fn publish(shared:&Shared,levels:[f32;BANDS]) {*shared.reading.lock().unwrap_or_else(|e|e.into_inner())=Reading{levels,at:Instant::now()};}

pub struct Visual {pub bars:[f32;BANDS],target:[f32;BANDS],last:Instant}
impl Visual {
    pub fn new()->Self {Self{bars:[0.;BANDS],target:[0.;BANDS],last:Instant::now()}}
    pub fn target(&mut self,reading:Reading,enabled:bool,now:Instant) {
        self.target=if enabled && now.saturating_duration_since(reading.at)<Duration::from_millis(250) {reading.levels}else{[0.;BANDS]};
    }
    pub fn tick(&mut self,now:Instant,motion:bool)->bool {
        let dt=now.saturating_duration_since(self.last).as_secs_f32().min(0.1);self.last=now;
        for (bar,target) in self.bars.iter_mut().zip(self.target) {
            let rate=if target>*bar {0.035}else{0.18};
            *bar=if motion {*bar+(target-*bar)*(1.-(-dt/rate).exp())}else{target};
            if (*bar-target).abs()<0.002 {*bar=target;}
        }self.moving()
    }
    pub fn moving(&self)->bool {self.bars.iter().zip(self.target).any(|(a,b)|(*a-b).abs()>0.002)}
    pub fn flat(&self)->bool {self.bars.iter().all(|v|*v==0.) && self.target.iter().all(|v|*v==0.)}
}

#[derive(Clone,Copy)]
struct Format {channels:usize,bytes:usize,stride:usize,float:bool,rate:u32}
impl Format {
    unsafe fn read(ptr:*const WAVEFORMATEX)->Option<Self> {
        let f=ptr.read_unaligned();let mut tag=f.wFormatTag;
        if tag==0xfffe {
            if f.cbSize<22 {return None;}
            let sub=std::ptr::addr_of!((*ptr.cast::<WAVEFORMATEXTENSIBLE>()).SubFormat).read_unaligned();
            tag=if sub==GUID::from_u128(0x00000003_0000_0010_8000_00aa00389b71) {3}
                else if sub==GUID::from_u128(0x00000001_0000_0010_8000_00aa00389b71) {1}else{return None;};
        }
        let bytes=f.wBitsPerSample as usize/8;let channels=f.nChannels as usize;
        if !(1..=8).contains(&channels) || f.nSamplesPerSec<8000 || f.wBitsPerSample as usize!=bytes*8
            || (f.nBlockAlign as usize)<channels*bytes || !((tag==3 && bytes==4)||(tag==1 && (1..=4).contains(&bytes))) {return None;}
        Some(Self{channels,bytes,stride:f.nBlockAlign as usize,float:tag==3,rate:f.nSamplesPerSec})
    }
    fn sample(self,data:&[u8])->f32 {
        let v=if self.float {f32::from_le_bytes(data[..4].try_into().unwrap())}else {match self.bytes {
            1=>(data[0] as f32-128.)/128.,
            2=>i16::from_le_bytes(data[..2].try_into().unwrap()) as f32/32768.,
            3=>((i32::from_le_bytes([0,data[0],data[1],data[2]]))>>8) as f32/8388608.,
            _=>i32::from_le_bytes(data[..4].try_into().unwrap()) as f32/2147483648.,
        }};if v.is_finite() {v.clamp(-1.,1.)}else{0.}
    }
}
unsafe fn default_device()->Result<(IMMDevice,String)> {
    let enumerator:IMMDeviceEnumerator=CoCreateInstance(&MMDeviceEnumerator,None,CLSCTX_ALL)?;
    let device=enumerator.GetDefaultAudioEndpoint(eRender,eConsole)?;let ptr=device.GetId()?;
    let id=ptr.to_string();CoTaskMemFree(Some(ptr.0.cast()));Ok((device,id?))
}
struct Stream {client:IAudioClient,capture:IAudioCaptureClient,id:String,format:Format,analyzer:Analyzer,last_packet:Instant,last_analysis:Instant}
impl Stream {
    unsafe fn open()->Result<Self> {
        let (device,id)=default_device()?;let client:IAudioClient=device.Activate(CLSCTX_ALL,None)?;
        let ptr=client.GetMixFormat()?;let format=Format::read(ptr);
        let initialized=if format.is_some() {client.Initialize(AUDCLNT_SHAREMODE_SHARED,AUDCLNT_STREAMFLAGS_LOOPBACK,1_000_000,0,ptr,None)}
            else{Err(windows::core::Error::from_hresult(windows::core::HRESULT(0x80004001u32 as i32)))};
        CoTaskMemFree(Some(ptr.cast()));initialized?;let format=format.unwrap();
        let capture=client.GetService()?;client.Start()?;
        Ok(Self{client,capture,id,format,analyzer:Analyzer::new(format.channels,format.rate),last_packet:Instant::now(),last_analysis:Instant::now()})
    }
    unsafe fn poll(&mut self,shared:&Shared)->Result<()> {
        // Bound draining so shutdown/device changes cannot be starved by a provider.
        for _ in 0..64 {
            if self.capture.GetNextPacketSize()?==0 {break;}
            let (mut data,mut frames,mut flags)=(std::ptr::null_mut(),0,0);
            self.capture.GetBuffer(&mut data,&mut frames,&mut flags,None,None)?;
            let silent=flags&AUDCLNT_BUFFERFLAGS_SILENT.0 as u32!=0 || data.is_null();
            if flags&AUDCLNT_BUFFERFLAGS_DATA_DISCONTINUITY.0 as u32!=0 {self.analyzer.reset();}
            for frame in 0..frames as usize {
                let mut values=[0.;8];
                if !silent {let bytes=std::slice::from_raw_parts(data.add(frame*self.format.stride),self.format.stride);
                    for (channel,value) in values.iter_mut().take(self.format.channels).enumerate() {*value=self.format.sample(&bytes[channel*self.format.bytes..]);}}
                self.analyzer.push(&values);
            }
            self.capture.ReleaseBuffer(frames)?;self.last_packet=Instant::now();
        }
        if self.last_analysis.elapsed()>=Duration::from_millis(33) {
            let levels=if self.last_packet.elapsed()>Duration::from_millis(150) {self.analyzer.reset();[0.;BANDS]}else{self.analyzer.levels()};
            publish(shared,levels);self.last_analysis=Instant::now();
        }Ok(())
    }
}
impl Drop for Stream {fn drop(&mut self) {unsafe {let _=self.client.Stop();}}}

struct Analyzer {ring:Vec<[f32;N]>,cursor:usize,filled:usize,rate:u32,real:[f32;N],imag:[f32;N],window:[f32;N]}
impl Analyzer {
    fn new(channels:usize,rate:u32)->Self {Self{ring:vec![[0.;N];channels],cursor:0,filled:0,rate,real:[0.;N],imag:[0.;N],
        window:std::array::from_fn(|i|0.5-0.5*(std::f32::consts::TAU*i as f32/(N-1) as f32).cos())}}
    fn reset(&mut self) {self.cursor=0;self.filled=0;for channel in &mut self.ring {channel.fill(0.);}}
    fn push(&mut self,values:&[f32;8]) {for (channel,v) in self.ring.iter_mut().zip(values) {channel[self.cursor]=*v;}
        self.cursor=(self.cursor+1)%N;self.filled=(self.filled+1).min(N);}
    fn levels(&mut self)->[f32;BANDS] {
        if self.filled<N {return [0.;BANDS];}
        let mut peaks=[0f32;BANDS];
        for channel in &self.ring {
            for i in 0..N {self.real[i]=channel[(self.cursor+i)%N]*self.window[i];}self.imag.fill(0.);
            fft(&mut self.real,&mut self.imag);
            for bin in 1..N/2 {
                let hz=bin as f32*self.rate as f32/N as f32;
                if let Some(band)=EDGES.windows(2).position(|edge|hz>=edge[0] && hz<edge[1]) {
                    // Independent channel power preserves anti-phase stereo and surround signals.
                    let amplitude=(self.real[bin].powi(2)+self.imag[bin].powi(2)).sqrt()*4./N as f32;
                    peaks[band]=peaks[band].max(amplitude);
                }
            }
        }
        peaks.map(|v|if v<0.0001 {0.}else{((20.*v.log10()+60.)/60.).clamp(0.,1.)})
    }
}
fn fft(real:&mut [f32;N],imag:&mut [f32;N]) {
    let mut j=0;
    for i in 1..N {let mut bit=N>>1;while j&bit!=0 {j^=bit;bit>>=1;}j^=bit;if i<j {real.swap(i,j);imag.swap(i,j);}}
    let mut size=2;
    while size<=N {
        let angle=-std::f32::consts::TAU/size as f32;let (sin,cos)=angle.sin_cos();
        for base in (0..N).step_by(size) {let (mut wr,mut wi)=(1.,0.);
            for offset in 0..size/2 {let a=base+offset;let b=a+size/2;
                let r=wr*real[b]-wi*imag[b];let im=wr*imag[b]+wi*real[b];
                real[b]=real[a]-r;imag[b]=imag[a]-im;real[a]+=r;imag[a]+=im;
                (wr,wi)=(wr*cos-wi*sin,wr*sin+wi*cos);
            }
        }size*=2;
    }
}

#[cfg(test)] mod tests {
    use super::*;
    #[test] fn sensitivity_changes_gain_without_inventing_audio_during_silence() {
        let reading=Reading{levels:[0.,0.2,0.5,0.8,1.],at:Instant::now()};
        assert_eq!(reading.sensitivity(100).levels,reading.levels);
        let lower=reading.sensitivity(50).levels;let higher=reading.sensitivity(200).levels;
        assert_eq!(lower[0],0.);assert_eq!(higher[0],0.);assert!(lower[2]<0.5 && higher[2]>0.5);assert_eq!(higher[4],1.);
    }
    #[test] fn silence_and_different_tones_produce_real_frequency_bands() {
        for (hz,band) in [(70.,0),(300.,1),(1000.,2),(5000.,3),(12000.,4)] {
            let mut analyzer=Analyzer::new(2,48000);assert_eq!(analyzer.levels(),[0.;BANDS]);
            for i in 0..N {let v=(std::f32::consts::TAU*hz*i as f32/48000.).sin()*0.5;
                analyzer.push(&[v,-v,0.,0.,0.,0.,0.,0.]);}
            let levels=analyzer.levels();let peak=levels.iter().enumerate().max_by(|a,b|a.1.total_cmp(b.1)).unwrap().0;
            assert_eq!(peak,band,"tone {hz}: {levels:?}");assert!(levels[band]>0.7);
            analyzer.reset();for _ in 0..N {analyzer.push(&[0.;8]);}assert_eq!(analyzer.levels(),[0.;BANDS]);
        }
    }
    #[test] fn pcm_float_decode_and_invalid_samples() {
        let mut f=Format{channels:2,bytes:3,stride:6,float:false,rate:48000};
        assert_eq!(f.sample(&[0,0,128]),-1.);assert!(f.sample(&[255,255,127])>0.99);
        f.bytes=2;assert_eq!(f.sample(&(-16384i16).to_le_bytes()),-0.5);
        f.float=true;f.bytes=4;assert_eq!(f.sample(&f32::NAN.to_le_bytes()),0.);assert_eq!(f.sample(&0.5f32.to_le_bytes()),0.5);
    }
    #[test] fn formats_validate_layout_and_extensible_subtypes() {
        let mut f=WAVEFORMATEXTENSIBLE{Format:WAVEFORMATEX{wFormatTag:0xfffe,nChannels:2,nSamplesPerSec:48000,nBlockAlign:8,wBitsPerSample:32,cbSize:22,..Default::default()},
            SubFormat:GUID::from_u128(0x00000003_0000_0010_8000_00aa00389b71),..Default::default()};
        unsafe {
            assert!(Format::read(&f.Format).unwrap().float);
            f.Format.cbSize=0;assert!(Format::read(&f.Format).is_none());
            f.Format.wFormatTag=1;f.Format.nBlockAlign=2;assert!(Format::read(&f.Format).is_none());
            f.Format.nBlockAlign=8;f.Format.nChannels=0;assert!(Format::read(&f.Format).is_none());
        }
    }
    #[test] fn stale_or_paused_audio_decays_and_settles() {
        let now=Instant::now();let mut visual=Visual::new();visual.last=now;
        let reading=Reading{levels:[1.;BANDS],at:now};visual.target(reading,true,now);
        visual.tick(now+Duration::from_millis(50),true);assert!(visual.bars[0]>0.5 && visual.bars[0]<1.);
        visual.target(reading,true,now+Duration::from_secs(1));
        for i in 1..40 {visual.tick(now+Duration::from_secs(1)+Duration::from_millis(i*50),true);}
        assert!(visual.flat());visual.target(reading,false,now);visual.tick(now,false);assert!(visual.flat());
    }
    #[test] #[ignore="opens live Windows output loopback; never starts playback or changes volume"]
    fn live_loopback_opens_and_polls() {
        unsafe {
            CoInitializeEx(None,COINIT_MULTITHREADED).ok().unwrap();
            let mut stream=Stream::open().expect("Default output must support shared-mode WASAPI loopback");
            let shared=Shared{enabled:AtomicBool::new(true),running:AtomicBool::new(true),reading:Mutex::new(Reading{levels:[0.;BANDS],at:Instant::now()})};
            println!("Loopback: {} channels, {} Hz, {} bytes/sample, float={}",stream.format.channels,stream.format.rate,stream.format.bytes,stream.format.float);
            let start=Instant::now();let mut peak=0f32;
            while start.elapsed()<Duration::from_secs(2) {
                stream.poll(&shared).expect("Loopback polling");
                let reading=*shared.reading.lock().unwrap();assert!(reading.levels.iter().all(|v|v.is_finite() && (0. ..=1.).contains(v)));
                peak=peak.max(reading.levels.into_iter().fold(0.,f32::max));std::thread::sleep(Duration::from_millis(16));
            }
            println!("Observed peak band: {peak:.3} (zero is valid when the output is silent)");
            drop(stream);CoUninitialize();
        }
    }
}
