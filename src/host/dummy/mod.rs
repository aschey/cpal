use std::{
    sync::mpsc::{self, Sender},
    thread::{self, JoinHandle},
};

use crate::{
    BuildStreamError, Data, DefaultStreamConfigError, DeviceNameError, DevicesError,
    InputCallbackInfo, OutputCallbackInfo, OutputStreamTimestamp, PauseStreamError,
    PlayStreamError, SampleFormat, SampleRate, StreamConfig, StreamError, StreamInstant,
    SupportedBufferSize, SupportedStreamConfig, SupportedStreamConfigRange,
    SupportedStreamConfigsError,
};
use traits::{DeviceTrait, HostTrait, StreamTrait};

#[derive(Default)]
pub struct Devices;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Device;

pub struct Host;

#[derive(Debug)]
pub struct Stream {
    audio_thread: Option<JoinHandle<()>>,
    sender: Option<Sender<()>>,
}

impl Drop for Stream {
    #[inline]
    fn drop(&mut self) {
        if let Some(sender) = self.sender.take() {
            sender.send(()).unwrap();
        }
        if let Some(thread) = self.audio_thread.take() {
            thread.join().unwrap();
        }
    }
}

pub struct SupportedInputConfigs;
pub struct SupportedOutputConfigs;

impl Host {
    #[allow(dead_code)]
    pub fn new() -> Result<Self, crate::HostUnavailable> {
        Ok(Host)
    }
}

impl Devices {
    pub fn new() -> Result<Self, DevicesError> {
        Ok(Devices)
    }
}

impl DeviceTrait for Device {
    type SupportedInputConfigs = SupportedInputConfigs;
    type SupportedOutputConfigs = SupportedOutputConfigs;
    type Stream = Stream;

    #[inline]
    fn name(&self) -> Result<String, DeviceNameError> {
        Ok("null".to_owned())
    }

    #[inline]
    fn supported_input_configs(
        &self,
    ) -> Result<SupportedInputConfigs, SupportedStreamConfigsError> {
        Ok(SupportedInputConfigs {})
    }

    #[inline]
    fn supported_output_configs(
        &self,
    ) -> Result<SupportedOutputConfigs, SupportedStreamConfigsError> {
        Ok(SupportedOutputConfigs {})
    }

    #[inline]
    fn default_input_config(&self) -> Result<SupportedStreamConfig, DefaultStreamConfigError> {
        Ok(SupportedStreamConfig {
            channels: 1,
            sample_rate: SampleRate(48000),
            buffer_size: SupportedBufferSize::Range {
                min: 0,
                max: u32::MAX,
            },
            sample_format: SampleFormat::F32,
        })
    }

    #[inline]
    fn default_output_config(&self) -> Result<SupportedStreamConfig, DefaultStreamConfigError> {
        Ok(SupportedStreamConfig {
            channels: 1,
            sample_rate: SampleRate(48000),
            buffer_size: SupportedBufferSize::Range {
                min: 0,
                max: u32::MAX,
            },
            sample_format: SampleFormat::F32,
        })
    }

    fn build_input_stream_raw<D, E>(
        &self,
        _config: &StreamConfig,
        _sample_format: SampleFormat,
        _data_callback: D,
        _error_callback: E,
    ) -> Result<Self::Stream, BuildStreamError>
    where
        D: FnMut(&Data, &InputCallbackInfo) + Send + 'static,
        E: FnMut(StreamError) + Send + 'static,
    {
        Ok(Self::Stream {
            audio_thread: None,
            sender: None,
        })
    }

    /// Create an output stream.
    fn build_output_stream_raw<D, E>(
        &self,
        _config: &StreamConfig,
        sample_format: SampleFormat,
        mut data_callback: D,
        _error_callback: E,
    ) -> Result<Self::Stream, BuildStreamError>
    where
        D: FnMut(&mut Data, &OutputCallbackInfo) + Send + 'static,
        E: FnMut(StreamError) + Send + 'static,
    {
        let (sender, receiver) = mpsc::channel();
        let handle = thread::spawn(move || {
            let mut buf = [0f32; 128];
            let buffer: &mut [f32] = &mut buf;
            let data = buffer.as_mut_ptr() as *mut ();
            let mut data = unsafe { Data::from_parts(data, 128, sample_format) };
            let info = OutputCallbackInfo {
                timestamp: OutputStreamTimestamp {
                    callback: StreamInstant { secs: 0, nanos: 0 },
                    playback: StreamInstant { secs: 0, nanos: 0 },
                },
            };
            loop {
                if let Ok(()) = receiver.try_recv() {
                    break;
                }
                data_callback(&mut data, &info);
            }
        });

        Ok(Self::Stream {
            audio_thread: Some(handle),
            sender: Some(sender),
        })
    }
}

impl HostTrait for Host {
    type Device = Device;
    type Devices = Devices;

    fn is_available() -> bool {
        true
    }

    fn devices(&self) -> Result<Self::Devices, DevicesError> {
        Devices::new()
    }

    fn default_input_device(&self) -> Option<Device> {
        Some(Device)
    }

    fn default_output_device(&self) -> Option<Device> {
        Some(Device {})
    }
}

impl StreamTrait for Stream {
    fn play(&self) -> Result<(), PlayStreamError> {
        Ok(())
    }

    fn pause(&self) -> Result<(), PauseStreamError> {
        Ok(())
    }
}

impl Iterator for Devices {
    type Item = Device;

    #[inline]
    fn next(&mut self) -> Option<Device> {
        None
    }
}

impl Iterator for SupportedInputConfigs {
    type Item = SupportedStreamConfigRange;

    #[inline]
    fn next(&mut self) -> Option<SupportedStreamConfigRange> {
        None
    }
}

impl Iterator for SupportedOutputConfigs {
    type Item = SupportedStreamConfigRange;

    #[inline]
    fn next(&mut self) -> Option<SupportedStreamConfigRange> {
        None
    }
}
