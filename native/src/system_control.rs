use std::{sync::mpsc::{self,Sender,Receiver},time::{Duration,Instant},ptr::null};
use windows::{core::{BSTR,VARIANT,w},Win32::{System::{Com::*,Wmi::*,Rpc::*},Media::Audio::{*,Endpoints::*}}};
use windows::Networking::Connectivity::*;

#[derive(Clone,Debug,Default,PartialEq,Eq)]
pub struct Audio {pub volume:Option<u8>,pub muted:bool}
#[derive(Clone,Debug,PartialEq,Eq)]
pub struct Devices {pub brightness:Option<u8>,pub wifi:String,pub wifi_connected:bool,pub bluetooth:String,pub connected:Vec<crate::bluetooth::Device>}
impl Default for Devices {fn default()->Self {Self{brightness:None,wifi:"Checking connection...".into(),wifi_connected:false,bluetooth:"Checking devices...".into(),connected:Vec::new()}}}
#[derive(Default)] pub struct Snapshot {pub audio:Audio,pub devices:Devices,pub error:Option<(String,Instant)>}
pub enum Update {Audio(Audio),Brightness(Option<u8>),Wifi(String,bool),Bluetooth(crate::bluetooth::Snapshot),Error(String)}
enum AudioCommand {Volume(u8),Mute(bool)}
enum DeviceCommand {Brightness(u8)}
pub struct Controller {audio:Sender<AudioCommand>,devices:Sender<DeviceCommand>,network:Sender<()>,bluetooth:Sender<()>}
impl Controller {
    pub fn volume(&self,value:u8) {let _=self.audio.send(AudioCommand::Volume(value.min(100)));}
    pub fn mute(&self,value:bool) {let _=self.audio.send(AudioCommand::Mute(value));}
    pub fn brightness(&self,value:u8) {let _=self.devices.send(DeviceCommand::Brightness(value.min(100)));}
    pub fn refresh(&self) {let _=self.network.send(());self.refresh_bluetooth();}
    pub fn refresh_bluetooth(&self) {let _=self.bluetooth.send(());}
}
pub unsafe fn audio_endpoint()->windows::core::Result<IAudioEndpointVolume> {
    let enumerator:IMMDeviceEnumerator=CoCreateInstance(&MMDeviceEnumerator,None,CLSCTX_ALL)?;
    enumerator.GetDefaultAudioEndpoint(eRender,eConsole)?.Activate(CLSCTX_ALL,None)
}
pub unsafe fn audio_read(endpoint:&IAudioEndpointVolume)->windows::core::Result<Audio> {
    Ok(Audio{volume:Some((endpoint.GetMasterVolumeLevelScalar()?.clamp(0.,1.)*100.).round() as u8),muted:endpoint.GetMute()?.as_bool()})
}

pub struct Brightness {services:IWbemServices,path:BSTR,signature:IWbemClassObject,instance:BSTR}
unsafe fn property(object:&IWbemClassObject,name:windows::core::PCWSTR)->windows::core::Result<VARIANT> {
    let mut value=VARIANT::new();object.Get(name,0,&mut value,None,None)?;Ok(value)
}
unsafe fn query(services:&IWbemServices,wql:&str)->windows::core::Result<Vec<IWbemClassObject>> {
    let values=services.ExecQuery(&BSTR::from("WQL"),&BSTR::from(wql),WBEM_FLAG_FORWARD_ONLY|WBEM_FLAG_RETURN_IMMEDIATELY,None)?;
    let mut result=Vec::new();
    for _ in 0..16 {let mut objects=[None];let mut count=0;let status=values.Next(200,&mut objects,&mut count);status.ok()?;
        if count==0 {if status.0==WBEM_S_TIMEDOUT.0 {continue;}break;} if let Some(object)=objects[0].take() {result.push(object);}}
    Ok(result)
}
impl Brightness {
    pub unsafe fn open()->windows::core::Result<Self> {
        let locator:IWbemLocator=CoCreateInstance(&WbemLocator,None,CLSCTX_INPROC_SERVER)?;
        let services=locator.ConnectServer(&BSTR::from("ROOT\\WMI"),&BSTR::new(),&BSTR::new(),&BSTR::new(),0,&BSTR::new(),None)?;
        CoSetProxyBlanket(&services,RPC_C_AUTHN_WINNT,RPC_C_AUTHZ_NONE,None,RPC_C_AUTHN_LEVEL_CALL,RPC_C_IMP_LEVEL_IMPERSONATE,None,EOAC_NONE)?;
        let invalid=|message:&str|windows::core::Error::new(windows::core::HRESULT(0x80004005u32 as i32),message);
        let monitor=query(&services,"SELECT * FROM WmiMonitorBrightness WHERE Active=TRUE")?.into_iter().next().ok_or_else(||invalid("No active WMI brightness display"))?;
        let instance=BSTR::try_from(&property(&monitor,w!("InstanceName"))?)?;
        let mut found=None;
        for method in query(&services,"SELECT * FROM WmiMonitorBrightnessMethods WHERE Active=TRUE")? {
            if BSTR::try_from(&property(&method,w!("InstanceName"))?)?==instance {found=Some(method);break;}
        }
        let method=found.ok_or_else(||invalid("No matching brightness method provider"))?;let path=BSTR::try_from(&property(&method,w!("__PATH"))?)?;
        let mut class=None;services.GetObject(&BSTR::from("WmiMonitorBrightnessMethods"),WBEM_GENERIC_FLAG_TYPE(0),None,Some(&mut class),None)?;
        let mut signature=None;class.ok_or_else(||invalid("Brightness class missing"))?.GetMethod(w!("WmiSetBrightness"),0,&mut signature,std::ptr::null_mut())?;
        Ok(Self{services,path,signature:signature.ok_or_else(||invalid("Brightness method signature missing"))?,instance})
    }
    pub unsafe fn read(&self)->windows::core::Result<u8> {
        for monitor in query(&self.services,"SELECT * FROM WmiMonitorBrightness WHERE Active=TRUE")? {
            if BSTR::try_from(&property(&monitor,w!("InstanceName"))?)?==self.instance {
                return Ok(u32::try_from(&property(&monitor,w!("CurrentBrightness"))?)?.min(100) as u8);
            }
        }
        Err(windows::core::Error::from_hresult(windows::core::HRESULT(0x80004005u32 as i32)))
    }
    pub unsafe fn set(&self,value:u8)->windows::core::Result<()> {
        let input=self.signature.SpawnInstance(0)?;
        // WMI's CIM integer properties use VT_I4 even for UInt8/UInt32 fields.
        input.Put(w!("Timeout"),0,&VARIANT::from(0i32),0)?;input.Put(w!("Brightness"),0,&VARIANT::from(value.min(100) as i32),0)?;
        let mut output=None;
        self.services.ExecMethod(&self.path,&BSTR::from("WmiSetBrightness"),WBEM_GENERIC_FLAG_TYPE(0),None,&input,Some(&mut output),None)?;
        if let Some(output)=output {let mut result=VARIANT::new();let mut kind=0;output.Get(w!("ReturnValue"),0,&mut result,Some(&mut kind),None)?;
            if kind!=CIM_BOOLEAN.0 && u32::try_from(&result).unwrap_or(1)!=0 {
            return Err(windows::core::Error::from_hresult(windows::core::HRESULT(0x80004005u32 as i32)));
        }}Ok(())
    }
}
fn wifi()->(String,bool) {
    let read=||->windows::core::Result<(String,bool)> {
        let profiles=NetworkInformation::GetConnectionProfiles()?;
        for profile in profiles {
            if profile.IsWlanConnectionProfile()? && profile.GetNetworkConnectivityLevel()?!=NetworkConnectivityLevel::None {
                let ssid=profile.WlanConnectionProfileDetails().and_then(|p|p.GetConnectedSsid()).or_else(|_|profile.ProfileName())?;
                return Ok((ssid.to_string(),true));
            }
        }Ok(("Not connected".into(),false))
    };read().unwrap_or_else(|_|("Wi-Fi unavailable".into(),false))
}

pub fn start()->(Controller,Receiver<Update>) {
    let (audio_tx,audio_rx)=mpsc::channel();let (devices_tx,devices_rx)=mpsc::channel();let (network_tx,network_rx)=mpsc::channel();let (updates,rx)=mpsc::channel();
    let bluetooth_tx=crate::bluetooth::start(updates.clone());
    let audio_out=updates.clone();
    std::thread::spawn(move || {unsafe {crate::initialize_runtime();}
        let mut endpoint=unsafe {audio_endpoint().ok()};let mut refresh=Instant::now();let mut last=None;let mut first=true;
        loop {
            let command=if first {first=false;None}else{match audio_rx.recv_timeout(Duration::from_millis(80)) {Ok(c)=>Some(c),Err(mpsc::RecvTimeoutError::Timeout)=>None,Err(_)=>return}};
            if refresh.elapsed()>=Duration::from_secs(2) {endpoint=unsafe {audio_endpoint().ok()};refresh=Instant::now();}
            let (mut volume,mut muted)=(None,None);for command in command.into_iter().chain(audio_rx.try_iter()) {match command {AudioCommand::Volume(v)=>volume=Some(v),AudioCommand::Mute(v)=>muted=Some(v)}}
            if volume.is_some() || muted.is_some() {
                let result=(||->windows::core::Result<()> {let endpoint=endpoint.as_ref().ok_or_else(||windows::core::Error::from_hresult(windows::core::HRESULT(0x80004005u32 as i32)))?;
                    unsafe {if let Some(v)=volume {endpoint.SetMasterVolumeLevelScalar(v as f32/100.,null())?;}if let Some(v)=muted {endpoint.SetMute(v,null())?;}}Ok(())})();
                if let Err(error)=result {let _=audio_out.send(Update::Error(format!("Volume: {error}")));last=None;}
            }
            let state=endpoint.as_ref().and_then(|endpoint|unsafe {audio_read(endpoint).ok()}).unwrap_or_default();
            if last.as_ref()!=Some(&state) {last=Some(state.clone());if audio_out.send(Update::Audio(state)).is_err() {return;}}
        }
    });
    let display_out=updates.clone();
    std::thread::spawn(move || {unsafe {crate::initialize_runtime();}
        let mut brightness=unsafe {Brightness::open().ok()};let mut reconnect=Instant::now();let mut last=None;let mut first=true;
        loop {
            let command=if first {first=false;None}else{match devices_rx.recv_timeout(Duration::from_millis(500)) {Ok(c)=>Some(c),Err(mpsc::RecvTimeoutError::Timeout)=>None,Err(_)=>return}};
            let mut desired=None;for c in command.into_iter().chain(devices_rx.try_iter()) {match c {DeviceCommand::Brightness(v)=>desired=Some(v)}}
            if brightness.is_none() && reconnect.elapsed()>Duration::from_secs(10) {brightness=unsafe {Brightness::open().ok()};reconnect=Instant::now();}
            if let Some(value)=desired {if let Some(control)=&brightness {if let Err(error)=unsafe {control.set(value)} {let _=display_out.send(Update::Error(format!("Brightness: {error}")));last=None;}}}
            let value=brightness.as_ref().and_then(|control|unsafe {control.read().ok()});
            if brightness.is_some() && value.is_none() {brightness=None;reconnect=Instant::now();}
            if last!=Some(value) {last=Some(value);if display_out.send(Update::Brightness(value)).is_err() {return;}}
        }
    });
    std::thread::spawn(move || {unsafe {crate::initialize_runtime();}
        let mut first=true;let mut last=None;
        loop {
            if first {first=false;}else {match network_rx.recv_timeout(Duration::from_secs(8)) {Ok(())|Err(mpsc::RecvTimeoutError::Timeout)=>{},Err(_)=>return}}
            while network_rx.try_recv().is_ok() {}
            let state=wifi();
            if last.as_ref()!=Some(&state) {last=Some(state.clone());if updates.send(Update::Wifi(state.0,state.1)).is_err() {return;}}
        }
    });
    (Controller{audio:audio_tx,devices:devices_tx,network:network_tx,bluetooth:bluetooth_tx},rx)
}

#[cfg(test)] mod tests {
    #[test] #[ignore="reads live Windows audio, display and connectivity"] fn live_system_status() {
        unsafe {crate::initialize_runtime();println!("Audio: {:?}",super::audio_endpoint().and_then(|a|super::audio_read(&a)));println!("Brightness: {:?}",super::Brightness::open().and_then(|b|b.read()));}
        println!("Wi-Fi: {:?}; Bluetooth: {:?}",super::wifi(),crate::bluetooth::read());
    }
    #[test] #[ignore="briefly adjusts and restores live system volume and brightness"] fn live_control_round_trip() {
        use super::*;
        unsafe {
            crate::initialize_runtime();let audio=audio_endpoint().unwrap();let volume=audio.GetMasterVolumeLevelScalar().unwrap();let muted=audio.GetMute().unwrap();
            let display=Brightness::open().unwrap();let brightness=display.read().unwrap();
            struct Restore {audio:IAudioEndpointVolume,volume:f32,muted:windows::Win32::Foundation::BOOL,display:Brightness,brightness:u8}
            impl Drop for Restore {fn drop(&mut self) {unsafe {let _=self.audio.SetMasterVolumeLevelScalar(self.volume,null());let _=self.audio.SetMute(self.muted,null());let _=self.display.set(self.brightness);}}}
            let restore=Restore{audio,volume,muted,display,brightness};
            let target=if volume<0.99 {volume+0.01}else{volume-0.01};restore.audio.SetMasterVolumeLevelScalar(target,null()).unwrap();
            assert!((restore.audio.GetMasterVolumeLevelScalar().unwrap()-target).abs()<0.005);
            let target=if brightness<99 {brightness+1}else{brightness-1};restore.display.set(target).unwrap();
            std::thread::sleep(Duration::from_millis(200));assert_eq!(restore.display.read().unwrap(),target);
            println!("Volume scalar round trip and brightness {brightness} -> {target} succeeded; restoring originals.");
        }
    }
}
