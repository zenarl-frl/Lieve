//! Connected-device status plus battery percentages cached by Windows drivers.
//! No pairing, disconnection, microphone access or vendor-protocol guesses.
use std::{collections::HashMap,sync::mpsc::{self,Sender},time::Duration};
use windows::{Devices::{Bluetooth::*,Enumeration::DeviceInformation,Radios::RadioState}};
use windows_sys::Win32::Devices::{DeviceAndDriverInstallation::*,Properties::*};

#[derive(Clone,Debug,PartialEq,Eq)]
pub struct Device {pub id:String,pub name:String,pub battery:Option<u8>}
#[derive(Clone,Debug,PartialEq,Eq)]
pub struct Snapshot {pub status:String,pub devices:Vec<Device>}

fn address_from_instance(instance:&str)->Option<u64> {
    let upper=instance.to_ascii_uppercase();
    for marker in ["BLUETOOTHDEVICE_","DEV_"] {
        for (start,_) in upper.match_indices(marker) {
            if let Some(hex)=upper.get(start+marker.len()..start+marker.len()+12) {
                if hex.bytes().all(|b|b.is_ascii_hexdigit()) {if let Ok(address)=u64::from_str_radix(hex,16) {return Some(address);}}
            }
        }
    }None
}
fn percent(bytes:&[u8],kind:u32)->Option<u8> {
    let value=match kind {
        DEVPROP_TYPE_BYTE=>*bytes.first()? as u32,
        DEVPROP_TYPE_UINT16=>u16::from_le_bytes(bytes.get(..2)?.try_into().ok()?) as u32,
        DEVPROP_TYPE_UINT32=>u32::from_le_bytes(bytes.get(..4)?.try_into().ok()?),
        _=>return None,
    };(value<=100).then_some(value as u8)
}
fn battery_catalog()->HashMap<u64,u8> {
    struct Set(HDEVINFO);impl Drop for Set {fn drop(&mut self) {unsafe {SetupDiDestroyDeviceInfoList(self.0);}}}
    let mut result=HashMap::new();
    unsafe {
        let handle=SetupDiGetClassDevsW(std::ptr::null(),std::ptr::null(),std::ptr::null_mut(),DIGCF_ALLCLASSES|DIGCF_PRESENT);
        if handle==-1 {return result;}let set=Set(handle);
        // Microsoft Bluetooth stack's cached battery percentage property.
        let key=DEVPROPKEY{fmtid:windows_sys::core::GUID::from_u128(0x104ea319_6ee2_4701_bd47_8ddbf425bbe5),pid:2};
        for index in 0..4096 {
            let mut device=std::mem::zeroed::<SP_DEVINFO_DATA>();device.cbSize=std::mem::size_of::<SP_DEVINFO_DATA>() as u32;
            if SetupDiEnumDeviceInfo(set.0,index,&mut device)==0 {break;}
            let mut id=[0u16;1024];
            if SetupDiGetDeviceInstanceIdW(set.0,&device,id.as_mut_ptr(),id.len() as u32,std::ptr::null_mut())==0 {continue;}
            let id=String::from_utf16_lossy(&id[..id.iter().position(|c|*c==0).unwrap_or(id.len())]);
            if !id.to_ascii_uppercase().starts_with("BTH") {continue;}
            let Some(address)=address_from_instance(&id) else{continue;};
            let mut kind=0;let mut buffer=[0u8;4];let mut size=buffer.len() as u32;
            if CM_Get_DevNode_PropertyW(device.DevInst,&key,&mut kind,buffer.as_mut_ptr(),&mut size,0)==CR_SUCCESS {
                if let Some(level)=percent(&buffer[..(size as usize).min(buffer.len())],kind) {result.insert(address,level);}
            }
        }
    }result
}
pub fn read()->Snapshot {
    let query=||->windows::core::Result<Snapshot> {
        let adapter=BluetoothAdapter::GetDefaultAsync()?.get()?;
        if adapter.GetRadioAsync().and_then(|op|op.get()).and_then(|r|r.State()).ok()==Some(RadioState::Off) {
            return Ok(Snapshot{status:"Bluetooth off".into(),devices:Vec::new()});
        }
        let batteries=battery_catalog();let mut devices=HashMap::<String,Device>::new();
        for (index,selector) in [BluetoothDevice::GetDeviceSelectorFromConnectionStatus(BluetoothConnectionStatus::Connected)?,
            BluetoothLEDevice::GetDeviceSelectorFromConnectionStatus(BluetoothConnectionStatus::Connected)?].into_iter().enumerate() {
            for device in DeviceInformation::FindAllAsyncAqsFilter(&selector)?.get()? {
                let id=device.Id()?;let name=device.Name()?.to_string();if name.trim().is_empty() {continue;}
                let address=if index==0 {BluetoothDevice::FromIdAsync(&id).and_then(|op|op.get()).and_then(|d|d.BluetoothAddress())}
                    else{BluetoothLEDevice::FromIdAsync(&id).and_then(|op|op.get()).and_then(|d|d.BluetoothAddress())}.ok().filter(|v|*v!=0)
                    .or_else(||address_from_instance(&id.to_string()));
                let key=address.map(|v|format!("{v:012x}")).unwrap_or_else(||id.to_string());
                let battery=address.and_then(|v|batteries.get(&v).copied());
                devices.entry(key.clone()).and_modify(|old|{if old.battery.is_none() {old.battery=battery;}}).or_insert(Device{id:key,name,battery});
            }
        }
        let mut devices:Vec<_>=devices.into_values().collect();devices.sort_by(|a,b|a.name.cmp(&b.name).then_with(||a.id.cmp(&b.id)));
        Ok(Snapshot{status:if devices.is_empty() {"None connected".into()}else{format!("{} connected",devices.len())},devices})
    };
    query().unwrap_or_else(|_|Snapshot{status:"Bluetooth unavailable".into(),devices:Vec::new()})
}
pub fn start(output:Sender<crate::system_control::Update>)->Sender<()> {
    let (tx,rx)=mpsc::channel();std::thread::spawn(move || {
        unsafe {crate::initialize_runtime();}let mut first=true;let mut last=None;
        loop {
            if first {first=false;}else{match rx.recv_timeout(Duration::from_secs(8)) {Ok(())|Err(mpsc::RecvTimeoutError::Timeout)=>{},Err(_)=>return}}
            while rx.try_recv().is_ok() {}
            let current=read();if last.as_ref()!=Some(&current) {last=Some(current.clone());if output.send(crate::system_control::Update::Bluetooth(current)).is_err() {return;}}
        }
    });tx
}
#[cfg(test)] mod tests {
    use super::*;
    #[test] fn battery_values_are_validated_not_clamped_into_fake_full_charge() {
        assert_eq!(percent(&[100],DEVPROP_TYPE_BYTE),Some(100));assert_eq!(percent(&[255],DEVPROP_TYPE_BYTE),None);
        assert_eq!(percent(&101u32.to_le_bytes(),DEVPROP_TYPE_UINT32),None);assert_eq!(percent(&53u32.to_le_bytes(),DEVPROP_TYPE_UINT32),Some(53));
        assert_eq!(percent(&[],DEVPROP_TYPE_BYTE),None);
        assert_eq!(address_from_instance("BTHLE\\DEV_C5B988D4CE7C\\instance"),Some(0xc5b988d4ce7c));
        assert_eq!(address_from_instance("BTHENUM\\service\\BLUETOOTHDEVICE_6468765B9FCA"),Some(0x6468765b9fca));
        assert_eq!(address_from_instance("BTHLEDEVICE\\service_DEV_VID&021532"),None);
    }
    #[test] #[ignore="reads real connected Bluetooth devices and driver-cached battery values"]
    fn live_connected_batteries() {
        unsafe {crate::initialize_runtime();}let snapshot=read();println!("Bluetooth: {}",snapshot.status);
        for device in &snapshot.devices {println!("{}: {:?}%",device.name,device.battery);assert!(device.battery.is_none_or(|v|v<=100));}
        println!("Cached Windows battery records: {}",battery_catalog().len());
    }
}
