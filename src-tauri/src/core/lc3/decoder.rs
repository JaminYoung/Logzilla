use super::ffi::{self, Lc3Library};

pub struct Lc3Decoder {
    lib: Lc3Library,
    decoders: Vec<*mut u8>,
    #[allow(dead_code)]
    decoder_mems: Vec<Vec<u8>>,
    num_channels: usize,
    frame_samples: usize,
    frame_bytes: usize,
}

unsafe impl Send for Lc3Decoder {}
unsafe impl Sync for Lc3Decoder {}

pub struct DecodeConfig {
    pub sample_rate: u32,
    pub num_channels: u32,
    pub bitrate: u32,
    pub frame_duration_ms: f64,
    pub hrmode: bool,
    pub libpath: Option<String>,
}

impl Lc3Decoder {
    pub fn new(config: &DecodeConfig) -> Result<Self, String> {
        let lib = lc3_load_library(config.libpath.as_deref())?;

        let dt_us = (config.frame_duration_ms * 1000.0) as i32;
        let sr_hz = config.sample_rate as i32;
        let nch = config.num_channels as i32;
        let bitrate = config.bitrate as i32;
        let hrmode = config.hrmode;

        if lib.frame_samples(hrmode, dt_us, sr_hz) < 0 {
            return Err("LC3 参数错误: 无效的帧时长/采样率".to_string());
        }
        if lib.frame_block_bytes(hrmode, dt_us, sr_hz, nch, bitrate) < 0 {
            return Err("LC3 参数错误: 无效的比特率".to_string());
        }

        let frame_samples = lib.frame_samples(hrmode, dt_us, sr_hz) as usize;
        let frame_bytes = lib.frame_block_bytes(hrmode, dt_us, sr_hz, nch, bitrate) as usize;

        let mut decoders = Vec::with_capacity(config.num_channels as usize);
        let mut decoder_mems = Vec::with_capacity(config.num_channels as usize);
        for _ in 0..config.num_channels as usize {
            let size = lib.decoder_size(hrmode, dt_us, sr_hz) as usize;
            let mut mem: Vec<u8> = vec![0u8; size];
            let ptr = mem.as_mut_ptr();
            let handle = lib.setup_decoder(hrmode, dt_us, sr_hz, sr_hz, ptr);
            decoders.push(handle);
            decoder_mems.push(mem);
        }

        Ok(Self {
            lib,
            decoders,
            decoder_mems,
            num_channels: config.num_channels as usize,
            frame_samples,
            frame_bytes,
        })
    }

    pub fn get_frame_samples(&self) -> usize {
        self.frame_samples
    }

    pub fn get_frame_bytes(&self) -> usize {
        self.frame_bytes
    }

    pub fn decode_frame(&self, data: &[u8]) -> Result<Vec<u8>, String> {
        let nch = self.num_channels;
        let pcm_len = nch * self.frame_samples;
        let mut pcm_buffer: Vec<u8> = vec![0u8; pcm_len * 2];

        let mut data_offset = 0;
        let data_len = data.len();

        for ich in 0..nch {
            let pcm_offset = ich * 2;
            let per_ch_data = data_len / nch + if ich < data_len % nch { 1 } else { 0 };

            if data_offset + per_ch_data > data_len {
                break;
            }

            let ch_data = &data[data_offset..data_offset + per_ch_data];
            data_offset += per_ch_data;

            let ret = self.lib.decode(
                self.decoders[ich],
                ch_data.as_ptr(),
                ch_data.len() as i32,
                ffi::PCM_FORMAT_S16,
                unsafe { pcm_buffer.as_mut_ptr().add(pcm_offset) },
                nch as i32,
            );

            if ret < 0 {
                return Err(format!("LC3 解码错误: channel {ich} 返回值 {ret}"));
            }
        }

        Ok(pcm_buffer)
    }

    #[allow(dead_code)]
    pub fn get_delay_samples(&self, hrmode: bool, dt_us: i32, sr_hz: i32) -> i32 {
        self.lib.delay_samples(hrmode, dt_us, sr_hz)
    }
}

pub fn lc3_load_library(libpath: Option<&str>) -> Result<Lc3Library, String> {
    if let Some(p) = libpath {
        if !p.is_empty() {
            return Lc3Library::load(p);
        }
    }

    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(dir) = exe_path.parent() {
            let p = dir.join("lc3.dll");
            if p.exists() {
                return Lc3Library::load(p);
            }
        }
    }

    let dev_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("lc3.dll");
    if dev_path.exists() {
        return Lc3Library::load(dev_path);
    }

    Err("找不到 lc3.dll，请确保与可执行文件放在同一目录".to_string())
}
