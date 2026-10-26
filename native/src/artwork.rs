use std::{sync::{Arc,mpsc::{self,Receiver,Sender}},time::{Duration,Instant}};
use windows::{core::Interface,Graphics::Imaging::*,Storage::Streams::*};

const EDGE:u32=256;
const MAX_BYTES:u64=8*1024*1024;
#[derive(Debug)]
pub struct Image {pub width:u32,pub height:u32,pub bgra:Vec<u8>}
pub struct Request {pub key:String,pub thumbnail:windows::core::AgileReference<IRandomAccessStreamReference>}
pub struct Update {pub key:String,pub image:Option<Arc<Image>>}

pub fn scaled_size(width:u32,height:u32)->Option<(u32,u32)> {
    if width==0 || height==0 || width>16384 || height>16384 || width as u64*height as u64>64*1024*1024 {return None;}
    let scale=(EDGE as f64/width.max(height) as f64).min(1.);
    Some(((width as f64*scale).round().max(1.) as u32,(height as f64*scale).round().max(1.) as u32))
}

pub fn decode(stream:&IRandomAccessStream)->windows::core::Result<Image> {
    let invalid=||windows::core::Error::from_hresult(windows::core::HRESULT(0x80070057u32 as i32));
    if stream.Size()?>MAX_BYTES {return Err(invalid());}
    let decoder=BitmapDecoder::CreateAsync(stream)?.get()?;
    let (width,height)=scaled_size(decoder.PixelWidth()?,decoder.PixelHeight()?).ok_or_else(invalid)?;
    let transform=BitmapTransform::new()?;
    transform.SetScaledWidth(width)?; transform.SetScaledHeight(height)?;
    transform.SetInterpolationMode(BitmapInterpolationMode::Fant)?;
    let bitmap=decoder.GetSoftwareBitmapTransformedAsync(BitmapPixelFormat::Bgra8,BitmapAlphaMode::Ignore,
        &transform,ExifOrientationMode::RespectExifOrientation,ColorManagementMode::DoNotColorManage)?.get()?;
    let (width,height)=(bitmap.PixelWidth()? as u32,bitmap.PixelHeight()? as u32);
    if width==0 || height==0 || width>EDGE || height>EDGE {return Err(invalid());}
    let bytes=width*height*4;
    let buffer=Buffer::Create(bytes)?;
    bitmap.CopyToBuffer(&buffer)?;
    let mut bgra=vec![0u8;bytes as usize];
    DataReader::FromBuffer(&buffer)?.ReadBytes(&mut bgra)?;
    Ok(Image{width,height,bgra})
}

pub fn load(thumbnail:&IRandomAccessStreamReference)->windows::core::Result<Image> {
    let stream=thumbnail.OpenReadAsync()?.get()?;
    decode(&stream.cast::<IRandomAccessStream>()?)
}

pub fn start()->(Sender<Request>,Receiver<Update>) {
    let (requests,input)=mpsc::channel::<Request>();
    let (output,updates)=mpsc::channel();
    std::thread::spawn(move || {
        unsafe {crate::initialize_runtime();}
        while let Ok(mut request)=input.recv() {
            // Drop superseded queued work before opening a provider's stream.
            while let Ok(latest)=input.try_recv() {request=latest;}
            let image=request.thumbnail.resolve().and_then(|thumbnail|load(&thumbnail)).ok().map(Arc::new);
            if output.send(Update{key:request.key,image}).is_err() {return;}
        }
    });
    (requests,updates)
}

pub struct Cache {key:String,pub image:Option<Arc<Image>>,pending:bool,next:Instant}
impl Cache {
    pub fn new()->Self {Self {key:String::new(),image:None,pending:false,next:Instant::now()}}
    pub fn track(&mut self,key:&str,now:Instant) {
        if self.key!=key {self.key=key.into();self.image=None;self.pending=false;self.next=now;}
    }
    pub fn due(&self,now:Instant)->bool {!self.pending && now>=self.next}
    pub fn pending(&self)->bool {self.pending}
    pub fn requested(&mut self) {self.pending=true;}
    pub fn missing(&mut self,now:Instant) {self.pending=false;self.next=now+Duration::from_secs(2);}
    pub fn apply(&mut self,update:Update,now:Instant) {
        if update.key!=self.key {return;}
        let success=update.image.is_some();
        if success {self.image=update.image;}
        self.pending=false;
        self.next=now+Duration::from_secs(if success {30}else{2});
    }
}

#[cfg(test)] mod tests {
    use super::*;
    #[test] fn bounded_decode_dimensions_preserve_aspect_ratio() {
        assert_eq!(scaled_size(1024,512),Some((256,128)));
        assert_eq!(scaled_size(32,64),Some((32,64)));
        assert_eq!(scaled_size(0,20),None); assert_eq!(scaled_size(u32::MAX,20),None);
    }
    #[test] fn track_changes_clear_old_covers_and_reject_stale_results() {
        let now=Instant::now();let mut cache=Cache::new();cache.track("A",now);cache.requested();
        let image=Arc::new(Image{width:1,height:1,bgra:vec![0,0,255,255]});
        cache.apply(Update{key:"A".into(),image:Some(image.clone())},now);
        assert!(cache.image.is_some());assert!(!cache.due(now));
        cache.apply(Update{key:"A".into(),image:None},now);
        assert!(Arc::ptr_eq(cache.image.as_ref().unwrap(),&image));
        cache.track("B",now);assert!(cache.image.is_none());assert!(cache.due(now));cache.requested();
        cache.apply(Update{key:"A".into(),image:Some(image)},now);
        assert!(cache.image.is_none());assert!(!cache.due(now));
        cache.apply(Update{key:"B".into(),image:None},now);
        assert!(!cache.due(now+Duration::from_secs(1)));assert!(cache.due(now+Duration::from_secs(2)));
    }
    #[test] fn windows_decoder_produces_top_down_bgra_and_rejects_bad_data() {
        unsafe {crate::initialize_runtime();}
        // Bottom-up 24-bit BMP: red/green bottom row, blue/white top row.
        let mut bmp=vec![0u8;70];bmp[..2].copy_from_slice(b"BM");
        bmp[2..6].copy_from_slice(&70u32.to_le_bytes());bmp[10..14].copy_from_slice(&54u32.to_le_bytes());
        bmp[14..18].copy_from_slice(&40u32.to_le_bytes());bmp[18..22].copy_from_slice(&2i32.to_le_bytes());
        bmp[22..26].copy_from_slice(&2i32.to_le_bytes());bmp[26..28].copy_from_slice(&1u16.to_le_bytes());
        bmp[28..30].copy_from_slice(&24u16.to_le_bytes());bmp[54..70].copy_from_slice(&[0,0,255,0,255,0,0,0,255,0,0,255,255,255,0,0]);
        let stream=InMemoryRandomAccessStream::new().unwrap();
        let writer=DataWriter::CreateDataWriter(&stream).unwrap();writer.WriteBytes(&bmp).unwrap();writer.StoreAsync().unwrap().get().unwrap();
        stream.Seek(0).unwrap();
        let image=decode(&stream.cast::<IRandomAccessStream>().unwrap()).unwrap();
        assert_eq!((image.width,image.height),(2,2));
        assert_eq!(&image.bgra[..3],&[255,0,0]);assert_eq!(&image.bgra[4..7],&[255,255,255]);
        assert_eq!(&image.bgra[8..11],&[0,0,255]);assert_eq!(&image.bgra[12..15],&[0,255,0]);
        let empty=InMemoryRandomAccessStream::new().unwrap();assert!(decode(&empty.cast::<IRandomAccessStream>().unwrap()).is_err());
    }
}
