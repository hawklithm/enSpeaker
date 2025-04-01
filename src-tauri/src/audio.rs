use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::SampleFormat;
use log::{error, info, warn};
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use std::env;
use std::sync::Arc;
use std::{fs::File, io::BufWriter, path::PathBuf};

static RECORDER: Lazy<Arc<Mutex<Option<AudioRecorder>>>> = Lazy::new(|| Arc::new(Mutex::new(None)));

pub struct AudioRecorder {
    stream: cpal::Stream,
    writer: Arc<Mutex<hound::WavWriter<BufWriter<File>>>>,
}

unsafe impl Send for AudioRecorder {}
unsafe impl Sync for AudioRecorder {}

fn write_input_data(input: &[f32], writer: &Arc<Mutex<hound::WavWriter<BufWriter<File>>>>) {
    for &sample in input {
        let normalized = sample;
        writer.lock().write_sample(normalized).unwrap();
    }
}

#[tauri::command]
pub async fn start_recording() -> Result<(), String> {
    let temp_dir = env::temp_dir();
    let file_name = format!(
        "audio_{}.wav",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis()
    );
    let file_path = temp_dir.join(file_name);
    info!("开始录音，文件将保存到: {:?}", file_path);

    let host = cpal::default_host();
    info!("音频主机类型: {:?}", host.id());

    // 列出所有可用的输入设备
    let input_devices = match host.input_devices() {
        Ok(devices) => devices,
        Err(e) => {
            error!("获取输入设备列表失败: {}", e);
            return Err("无法获取输入设备列表".to_string());
        }
    };

    info!("可用的输入设备列表:");
    for device in input_devices {
        info!(
            "设备名称: {:?}",
            device.name().unwrap_or_else(|_| "未知设备名".into())
        );
    }

    let device = match host.default_input_device() {
        Some(dev) => {
            info!(
                "成功获取默认输入设备: {:?}",
                dev.name().unwrap_or_else(|_| "未知设备名".into())
            );
            dev
        }
        None => {
            error!("未找到默认录音设备，尝试获取第一个可用设备");
            host.input_devices()
                .map_err(|e| {
                    error!("获取输入设备列表失败: {}", e);
                    e.to_string()
                })?
                .next()
                .ok_or_else(|| {
                    error!("系统中没有任何可用的录音设备");
                    "没有找到任何录音设备".to_string()
                })?
        }
    };

    // 获取设备支持的配置
    info!("尝试获取设备支持的配置");
    let supported_configs = device.supported_input_configs().map_err(|e| {
        error!("获取设备支持的配置失败: {}", e);
        e.to_string()
    })?;

    info!("设备支持的配置列表:");
    for config in supported_configs {
        info!(
            "采样率范围: {:?}-{:?}Hz, 声道数: {}, 格式: {:?}",
            config.min_sample_rate().0,
            config.max_sample_rate().0,
            config.channels(),
            config.sample_format()
        );
    }

    let config = device.default_input_config().map_err(|e| e.to_string())?;
    info!("录音配置: {:?}", config);

    let spec = hound::WavSpec {
        channels: config.channels(),
        sample_rate: config.sample_rate().0,
        bits_per_sample: 32,
        sample_format: hound::SampleFormat::Float,
    };

    let file = BufWriter::new(File::create(PathBuf::from(file_path)).map_err(|e| e.to_string())?);
    let writer = hound::WavWriter::new(file, spec).map_err(|e| e.to_string())?;
    let writer = Arc::new(Mutex::new(writer));
    let writer_clone = writer.clone();

    let stream = match config.sample_format() {
        cpal::SampleFormat::F32 => device.build_input_stream(
            &config.into(),
            move |data: &[f32], _: &_| write_input_data(data, &writer_clone),
            |err| eprintln!("录音错误: {}", err),
            None,
        ),
        _ => return Err("不支持的音频格式".to_string()),
    }
    .map_err(|e| e.to_string())?;

    stream.play().map_err(|e| e.to_string())?;
    info!("录音流已启动");

    let recorder = AudioRecorder { stream, writer };

    *RECORDER.lock() = Some(recorder);
    info!("录音器已初始化并开始录音");
    Ok(())
}

#[tauri::command]
pub async fn stop_recording() -> Result<String, String> {
    info!("准备停止录音");

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

        info!("录音已完成");
        Ok("录音已完成".to_string())
    } else {
        warn!("没有找到正在进行的录音");
        Err("没有正在进行的录音".to_string())
    }
}
