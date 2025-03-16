use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use std::sync::Arc;
use std::{fs::File, io::BufWriter, path::PathBuf};
use std::env;
use log::{info, error, warn};

static RECORDER: Lazy<Arc<Mutex<Option<AudioRecorder>>>> = Lazy::new(|| Arc::new(Mutex::new(None)));

pub struct AudioRecorder {
    stream: cpal::Stream,
    writer: Arc<Mutex<hound::WavWriter<BufWriter<File>>>>,
}

unsafe impl Send for AudioRecorder {}
unsafe impl Sync for AudioRecorder {}

#[tauri::command]
pub async fn start_recording() -> Result<(), String> {
    let temp_dir = env::temp_dir();
    let file_name = format!("audio_{}.wav", std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis());
    let file_path = temp_dir.join(file_name);
    info!("开始录音，文件将保存到: {:?}", file_path);

    let host = cpal::default_host();
    let device = host.default_input_device()
        .ok_or_else(|| {
            error!("未找到录音设备");
            "没有找到录音设备".to_string()
        })?;
    info!("使用录音设备: {:?}", device.name());

    let config = device.default_input_config()
        .map_err(|e| e.to_string())?;
    info!("录音配置: {:?}", config);

    let spec = hound::WavSpec {
        channels: config.channels(),
        sample_rate: config.sample_rate().0,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    let file = BufWriter::new(File::create(PathBuf::from(file_path))
        .map_err(|e| e.to_string())?);
    let writer = hound::WavWriter::new(file, spec)
        .map_err(|e| e.to_string())?;
    let writer = Arc::new(Mutex::new(writer));
    let writer_clone = writer.clone();

    let stream = match config.sample_format() {
        cpal::SampleFormat::I16 => device.build_input_stream(
            &config.into(),
            move |data: &[i16], _: &_| {
                for &sample in data {
                    writer_clone.lock().write_sample(sample).unwrap();
                }
            },
            |err| eprintln!("录音错误: {}", err),
            None,
        ),
        _ => return Err("不支持的音频格式".to_string()),
    }.map_err(|e| e.to_string())?;

    stream.play().map_err(|e| e.to_string())?;
    info!("录音流已启动");

    let recorder = AudioRecorder {
        stream,
        writer,
    };

    *RECORDER.lock() = Some(recorder);
    info!("录音器已初始化并开始录音");
    Ok(())
}

#[tauri::command]
pub async fn stop_recording() -> Result<String, String> {
    let temp_dir = env::temp_dir();
    let file_name = format!("audio_{}.wav", std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis());
    let file_path = temp_dir.join(file_name);
    info!("准备停止录音，文件路径: {:?}", file_path);

    if let Some(recorder) = RECORDER.lock().take() {
        info!("正在停止录音流");
        drop(recorder.stream);
        let _writer = Arc::try_unwrap(recorder.writer)
            .map_err(|_| {
                error!("无法获取writer所有权");
                "无法获取writer所有权".to_string()
            })?
            .into_inner()
            .finalize()
            .map_err(|e| e.to_string())?;

        info!("录音已完成并保存到: {:?}", file_path);
        Ok(file_path.to_string_lossy().into_owned())
    } else {
        warn!("没有找到正在进行的录音");
        Err("没有正在进行的录音".to_string())
    }
} 