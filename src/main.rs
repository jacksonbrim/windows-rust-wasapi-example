use std::time::{Duration, Instant};
use windows::{
    core::*,
    Win32::{Devices::FunctionDiscovery::*, Media::Audio::*, System::Com::*},
};
pub mod tone;

use crate::tone::ToneGenerator;
fn main() -> windows::core::Result<()> {
    unsafe {
        CoInitializeEx(None, COINIT_MULTITHREADED).ok()?;
        let device = get_output_device()?;
        print_device_name(&device)?;
        play_audio_stream(&device)?;
    }
    Ok(())
}

unsafe fn print_input_devices() -> Result<()> {
    let device_enumerator: IMMDeviceEnumerator =
        CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)?;
    let devices = device_enumerator.EnumAudioEndpoints(eCapture, DEVICE_STATE_ACTIVE)?;
    let num_devices = devices.GetCount()?;
    for device_idx in 0..num_devices {
        println!("device #{}", device_idx);
        let device = devices.Item(device_idx)?;
        let property_store = device.OpenPropertyStore(STGM_READ)?;
        let format = property_store.GetValue(&PKEY_AudioEngine_DeviceFormat as *const _)?;
        let name = property_store.GetValue(&PKEY_Device_FriendlyName as *const _)?;
        let name = name.to_string();
        let format = format.to_string();
        println!("device name: {}", &name);
        println!("device format: {}", &format);
    }
    Ok(())
}

unsafe fn get_output_device() -> Result<IMMDevice> {
    let device_enumerator: IMMDeviceEnumerator =
        CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)?;
    let device = device_enumerator.GetDefaultAudioEndpoint(eRender, eConsole)?;
    Ok(device)
}

unsafe fn get_input_device() -> Result<IMMDevice> {
    let device_enumerator: IMMDeviceEnumerator =
        CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)?;
    let device = device_enumerator.GetDefaultAudioEndpoint(eCapture, eConsole)?;
    Ok(device)
}

unsafe fn print_device_name(device: &IMMDevice) -> Result<()> {
    let property_store = device.OpenPropertyStore(STGM_READ)?;
    let format = property_store.GetValue(&PKEY_AudioEngine_DeviceFormat as *const _)?;
    let name = property_store.GetValue(&PKEY_Device_FriendlyName as *const _)?;
    let name = name.to_string();
    let format = format.to_string();
    println!("device name: {}", &name);
    println!("device format: {}", &format);
    Ok(())
}

unsafe fn play_audio_stream(device: &IMMDevice) -> Result<()> {
    let audio_client: IAudioClient = device.Activate(CLSCTX_ALL, None)?;
    let wave_format: *const WAVEFORMATEX = audio_client.GetMixFormat()?;
    let sample_rate = (*wave_format).nSamplesPerSec;
    let channels = (*wave_format).nChannels;
    let block_align = (*wave_format).nBlockAlign;
    let bits_per_sample = (*wave_format).wBitsPerSample;

    println!("sample rate: {}", sample_rate);
    println!("channels: {}", channels);
    println!("block align: {}", block_align);
    println!("bits per sample: {}", bits_per_sample);

    let mut generator = ToneGenerator::new(*wave_format, 440., 0.5);
    let mut elapsed = Duration::from_secs(0);
    let mut def_period = 0i64;
    let mut min_period = 0i64;
    audio_client.GetDevicePeriod(Some(&mut def_period), Some(&mut min_period))?;
    let start = Instant::now();

    audio_client.Initialize(
        AUDCLNT_SHAREMODE_SHARED,
        0,
        def_period,
        0,
        wave_format,
        None,
    )?;

    println!("start audio output");
    let buffer_frame_count = audio_client.GetBufferSize()?;
    println!("Buffer size: {}", buffer_frame_count);
    if buffer_frame_count == 0 {
        println!("Buffer size is 0 frames.");
        return Err(windows::core::Error::from_win32());
    }

    let render_client: IAudioRenderClient = audio_client.GetService()?;
    audio_client.Start()?;

    while elapsed.as_millis() < 3000 {
        std::thread::sleep(Duration::from_millis(500));
        let num_frames_padding = audio_client.GetCurrentPadding()?;
        let buffer_size = audio_client.GetBufferSize()?;
        let num_frames_available = buffer_size - num_frames_padding;
        if num_frames_available > 0 {
            let device_buffer = render_client.GetBuffer(num_frames_available)?;

            generator.fill_buffer(device_buffer, num_frames_available as usize);
            render_client.ReleaseBuffer(num_frames_available, 0)?;
        }
        elapsed = start.elapsed();
    }

    println!("stop audio output");
    audio_client.Stop()?;

    Ok(())
}
