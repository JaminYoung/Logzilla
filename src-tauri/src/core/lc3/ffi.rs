use std::path::Path;
use libloading::Library;

pub const PCM_FORMAT_S16: i32 = 0;

type FnFrameSamples = unsafe extern "C" fn(i32, i32) -> i32;
type FnFrameBytes = unsafe extern "C" fn(i32, i32) -> i32;
type FnDelaySamples = unsafe extern "C" fn(i32, i32) -> i32;
type FnDecoderSize = unsafe extern "C" fn(i32, i32) -> u32;
type FnSetupDecoder = unsafe extern "C" fn(i32, i32, i32, *mut u8) -> *mut u8;
type FnDecode = unsafe extern "C" fn(*mut u8, *const u8, i32, i32, *mut u8, i32) -> i32;

type FnHrFrameSamples = unsafe extern "C" fn(i32, i32, i32) -> i32;
type FnHrFrameBlockBytes = unsafe extern "C" fn(i32, i32, i32, i32, i32) -> i32;
type FnHrDelaySamples = unsafe extern "C" fn(i32, i32, i32) -> i32;
type FnHrDecoderSize = unsafe extern "C" fn(i32, i32, i32) -> u32;
type FnHrSetupDecoder = unsafe extern "C" fn(i32, i32, i32, i32, *mut u8) -> *mut u8;

fn load_fn<T: Clone>(lib: &Library, name: &[u8]) -> Option<T> {
    unsafe { lib.get::<T>(name).ok().map(|s| (*s).clone()) }
}

pub struct Lc3Library {
    _lib: Library,
    _has_hr: bool,

    frame_samples: FnFrameSamples,
    frame_bytes: FnFrameBytes,
    delay_samples: FnDelaySamples,
    decoder_size: FnDecoderSize,
    setup_decoder: FnSetupDecoder,
    decode: FnDecode,

    hr_frame_samples: Option<FnHrFrameSamples>,
    hr_frame_block_bytes: Option<FnHrFrameBlockBytes>,
    hr_delay_samples: Option<FnHrDelaySamples>,
    hr_decoder_size: Option<FnHrDecoderSize>,
    hr_setup_decoder: Option<FnHrSetupDecoder>,
}

impl Lc3Library {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        unsafe {
            let lib = Library::new(path.as_ref())
                .map_err(|e| format!("加载 lc3.dll 失败: {}", e))?;

            let frame_samples: FnFrameSamples = load_fn(&lib, b"lc3_frame_samples")
                .ok_or_else(|| "找不到 lc3_frame_samples".to_string())?;
            let frame_bytes: FnFrameBytes = load_fn(&lib, b"lc3_frame_bytes")
                .ok_or_else(|| "找不到 lc3_frame_bytes".to_string())?;
            let delay_samples: FnDelaySamples = load_fn(&lib, b"lc3_delay_samples")
                .ok_or_else(|| "找不到 lc3_delay_samples".to_string())?;
            let decoder_size: FnDecoderSize = load_fn(&lib, b"lc3_decoder_size")
                .ok_or_else(|| "找不到 lc3_decoder_size".to_string())?;
            let setup_decoder: FnSetupDecoder = load_fn(&lib, b"lc3_setup_decoder")
                .ok_or_else(|| "找不到 lc3_setup_decoder".to_string())?;
            let decode: FnDecode = load_fn(&lib, b"lc3_decode")
                .ok_or_else(|| "找不到 lc3_decode".to_string())?;

            let has_hr = load_fn::<FnHrFrameSamples>(&lib, b"lc3_hr_frame_samples").is_some();

            Ok(Self {
                hr_frame_samples: load_fn(&lib, b"lc3_hr_frame_samples"),
                hr_frame_block_bytes: load_fn(&lib, b"lc3_hr_frame_block_bytes"),
                hr_delay_samples: load_fn(&lib, b"lc3_hr_delay_samples"),
                hr_decoder_size: load_fn(&lib, b"lc3_hr_decoder_size"),
                hr_setup_decoder: load_fn(&lib, b"lc3_hr_setup_decoder"),
                _lib: lib,
                _has_hr: has_hr,
                frame_samples,
                frame_bytes,
                delay_samples,
                decoder_size,
                setup_decoder,
                decode,
            })
        }
    }

    pub fn frame_samples(&self, hrmode: bool, dt_us: i32, sr_hz: i32) -> i32 {
        unsafe {
            if hrmode {
                if let Some(f) = self.hr_frame_samples {
                    f(hrmode as i32, dt_us, sr_hz)
                } else {
                    (self.frame_samples)(dt_us, sr_hz)
                }
            } else {
                (self.frame_samples)(dt_us, sr_hz)
            }
        }
    }

    pub fn frame_block_bytes(
        &self,
        hrmode: bool,
        dt_us: i32,
        sr_hz: i32,
        nch: i32,
        bitrate: i32,
    ) -> i32 {
        unsafe {
            if let Some(f) = self.hr_frame_block_bytes {
                f(hrmode as i32, dt_us, sr_hz, nch, bitrate)
            } else {
                nch * (self.frame_bytes)(dt_us, bitrate / 2)
            }
        }
    }

    pub fn delay_samples(&self, hrmode: bool, dt_us: i32, sr_hz: i32) -> i32 {
        unsafe {
            if let Some(f) = self.hr_delay_samples {
                f(hrmode as i32, dt_us, sr_hz)
            } else {
                (self.delay_samples)(dt_us, sr_hz)
            }
        }
    }

    pub fn decoder_size(&self, hrmode: bool, dt_us: i32, sr_hz: i32) -> u32 {
        unsafe {
            if let Some(f) = self.hr_decoder_size {
                f(hrmode as i32, dt_us, sr_hz)
            } else {
                (self.decoder_size)(dt_us, sr_hz)
            }
        }
    }

    pub fn setup_decoder(
        &self,
        hrmode: bool,
        dt_us: i32,
        sr_hz: i32,
        sr_pcm_hz: i32,
        mem: *mut u8,
    ) -> *mut u8 {
        unsafe {
            if let Some(f) = self.hr_setup_decoder {
                f(hrmode as i32, dt_us, sr_hz, sr_pcm_hz, mem)
            } else {
                (self.setup_decoder)(dt_us, sr_hz, sr_pcm_hz, mem)
            }
        }
    }

    pub fn decode(&self, decoder: *mut u8, data: *const u8, data_size: i32, pcm_format: i32, out: *mut u8, stride: i32) -> i32 {
        unsafe { (self.decode)(decoder, data, data_size, pcm_format, out, stride) }
    }
}
