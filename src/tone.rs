use std::sync::{Arc, Mutex};
use windows::core::GUID;
use windows::Win32::Media::Audio::{WAVEFORMATEX, WAVEFORMATEXTENSIBLE};
use windows::Win32::Media::KernelStreaming::KSDATAFORMAT_SUBTYPE_PCM;
use windows::Win32::Media::Multimedia::KSDATAFORMAT_SUBTYPE_IEEE_FLOAT;

pub struct ToneGenerator {
    pub wave_format: WAVEFORMATEX,
    pub amplitude: f32,
    pub sample_rate: f32,
    pub frequencies: Arc<Mutex<Vec<f32>>>,
    pub phases: Arc<Mutex<Vec<f32>>>,
}
impl ToneGenerator {
    pub fn new(wave_format: WAVEFORMATEX, frequency: f32, amplitude: f32) -> Self {
        let sample_rate = wave_format.nSamplesPerSec as f32;
        ToneGenerator {
            wave_format,
            amplitude,
            sample_rate,
            frequencies: Arc::new(Mutex::new(vec![frequency])),
            phases: Arc::new(Mutex::new(vec![0.0])),
        }
    }

    fn cmp_guid(a: &GUID, b: &GUID) -> bool {
        (a.data1, a.data2, a.data3, a.data4) == (b.data1, b.data2, b.data3, b.data4)
    }

    fn get_wave_format_tag(&self) -> &str {
        match (
            self.wave_format.wBitsPerSample,
            self.wave_format.wFormatTag as u32,
        ) {
            (8, WAVE_FORMAT_PCM) => "u8",
            (16, WAVE_FORMAT_PCM) => "u16",
            (32, WAVE_FORMAT_IEEE_FLOAT) => "F32",
            (n_bits, WAVE_FORMAT_EXTENSIBLE) => {
                let waveformatextensible_ptr =
                    &self.wave_format as *const _ as *const WAVEFORMATEXTENSIBLE;
                let sub = unsafe { (*waveformatextensible_ptr).SubFormat };

                if Self::cmp_guid(&sub, &KSDATAFORMAT_SUBTYPE_PCM) {
                    match n_bits {
                        8 => "u8",
                        16 => "I16",
                        32 => "I32",
                        64 => "I64",
                        _ => "unknown wave format",
                    }
                } else if n_bits == 32 && Self::cmp_guid(&sub, &KSDATAFORMAT_SUBTYPE_IEEE_FLOAT) {
                    "F32"
                } else {
                    "unknown wave format"
                }
            }
            _ => "unknown wave format type",
        }
    }

    pub fn fill_buffer(&mut self, buffer: *mut u8, num_frames: usize) {
        assert!(!buffer.is_null(), "Buffer pointer is null.");

        let format_tag = self.get_wave_format_tag();
        match format_tag {
            "u8" => self.fill_buffer_u8(buffer, num_frames),
            "u16" => self.fill_buffer_u16(buffer, num_frames),
            "I16" => self.fill_buffer_i16(buffer, num_frames),
            "F32" => self.fill_buffer_f32(buffer, num_frames),
            _ => panic!("Unsupported wave format"),
        }
    }

    fn fill_buffer_u8(&mut self, buffer: *mut u8, num_frames: usize) {
        let buffer_slice = unsafe { std::slice::from_raw_parts_mut(buffer, num_frames) };
        for i in 0..num_frames {
            let sample = ((self.next_sample() * self.amplitude * 127.0) + 128.0) as u8;
            buffer_slice[i] = sample;
        }
    }

    fn fill_buffer_u16(&mut self, buffer: *mut u8, num_frames: usize) {
        let buffer_slice =
            unsafe { std::slice::from_raw_parts_mut(buffer as *mut u16, num_frames) };
        for i in 0..num_frames {
            let sample = ((self.next_sample() * self.amplitude * i16::MAX as f32) as u16);
            buffer_slice[i] = sample;
        }
    }

    fn fill_buffer_i16(&mut self, buffer: *mut u8, num_frames: usize) {
        let buffer_slice =
            unsafe { std::slice::from_raw_parts_mut(buffer as *mut i16, num_frames) };
        for i in 0..num_frames {
            let sample = ((self.next_sample() * self.amplitude * i16::MAX as f32) as i16);
            buffer_slice[i] = sample;
        }
    }

    fn fill_buffer_f32(&mut self, buffer: *mut u8, num_frames: usize) {
        let buffer_slice =
            unsafe { std::slice::from_raw_parts_mut(buffer as *mut f32, num_frames) };
        for i in 0..num_frames {
            let sample = self.next_sample() * self.amplitude;
            buffer_slice[i] = sample;
        }
    }
    pub fn next_sample(&mut self) -> f32 {
        let frequencies = self.frequencies.lock().unwrap();
        let mut phases = self.phases.lock().unwrap();
        let mut value = 0.0;
        let num_frequencies = frequencies.len();

        for (i, &frequency) in frequencies.iter().enumerate() {
            let increment = frequency * 2.0 * std::f32::consts::PI / self.sample_rate;

            // Generate the waveform with multiple harmonics
            let fundamental = phases[i].sin();
            let harmonic1 = (phases[i] * 2.0).sin() * 0.5;
            let harmonic2 = (phases[i] * 3.0).sin() * 0.25;
            let harmonic3 = (phases[i] * 4.0).sin() * 0.125;

            // Sum the harmonics to create a richer tone
            value += (fundamental + harmonic1 + harmonic2 + harmonic3) / num_frequencies as f32;

            // Update phase for the next sample
            phases[i] += increment;
            if phases[i] >= 2.0 * std::f32::consts::PI {
                phases[i] -= 2.0 * std::f32::consts::PI;
            }
        }

        value * 0.6
    }
}
