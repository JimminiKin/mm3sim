use bevy::prelude::*;

use crate::resources::constants::VIB_BAR_COUNT;
use crate::systems::instrument::{InstrumentHits, CH_SNARE, CH_VIB_FIRST};

const MAX_IMPACT_SPEED: f32 = 4.0;

#[derive(Resource)]
pub struct SnareVolume(pub f32);

impl Default for SnareVolume {
    fn default() -> Self { SnareVolume(0.5) }
}

fn impact_volume(speed: f32, vol: f32) -> f32 {
    (speed / MAX_IMPACT_SPEED).clamp(0.0, 1.0).powf(0.5) * vol
}

// ── Native ────────────────────────────────────────────────────────────────────

#[cfg(not(target_arch = "wasm32"))]
use std::sync::Arc;

#[cfg(not(target_arch = "wasm32"))]
use bevy::audio::Volume;

#[cfg(not(target_arch = "wasm32"))]
#[derive(Resource)]
pub(crate) struct SnareHitSound(Handle<AudioSource>);

#[cfg(not(target_arch = "wasm32"))]
#[derive(Resource)]
pub(crate) struct VibHitSounds(Vec<Handle<AudioSource>>);

#[cfg(not(target_arch = "wasm32"))]
pub fn setup_sounds(mut audio_sources: ResMut<Assets<AudioSource>>, mut commands: Commands) {
    let snare = audio_sources.add(AudioSource { bytes: Arc::from(generate_snare_wav()) });
    commands.insert_resource(SnareHitSound(snare));

    let vib: Vec<_> = (0..VIB_BAR_COUNT)
        .map(|i| audio_sources.add(AudioSource { bytes: Arc::from(generate_vib_wav(i)) }))
        .collect();
    commands.insert_resource(VibHitSounds(vib));
}

#[cfg(not(target_arch = "wasm32"))]
pub fn play_instrument_sounds(
    hits: Res<InstrumentHits>,
    snare_sound: Option<Res<SnareHitSound>>,
    vib_sounds: Option<Res<VibHitSounds>>,
    volume: Res<SnareVolume>,
    mut commands: Commands,
) {
    for hit in &hits.0 {
        let vol = impact_volume(hit.speed, volume.0);
        if hit.channel == CH_SNARE {
            let Some(s) = snare_sound.as_ref() else { continue };
            commands.spawn((
                AudioPlayer(s.0.clone()),
                PlaybackSettings { volume: Volume::Linear(vol), ..PlaybackSettings::ONCE },
            ));
        } else if hit.channel >= CH_VIB_FIRST {
            let Some(sounds) = vib_sounds.as_ref() else { continue };
            let bar_idx = hit.channel - CH_VIB_FIRST;
            let Some(handle) = sounds.0.get(bar_idx) else { continue };
            commands.spawn((
                AudioPlayer(handle.clone()),
                PlaybackSettings { volume: Volume::Linear(vol), ..PlaybackSettings::ONCE },
            ));
        }
    }
}

// ── WASM ──────────────────────────────────────────────────────────────────────

#[cfg(target_arch = "wasm32")]
thread_local! {
    static AUDIO_CTX: std::cell::RefCell<Option<web_sys::AudioContext>> =
        std::cell::RefCell::new(None);
}

#[cfg(target_arch = "wasm32")]
pub fn setup_sounds() {
    AUDIO_CTX.with(|slot| { *slot.borrow_mut() = web_sys::AudioContext::new().ok(); });
}

#[cfg(target_arch = "wasm32")]
pub fn play_instrument_sounds(hits: Res<InstrumentHits>, volume: Res<SnareVolume>) {
    for hit in &hits.0 {
        let vol = impact_volume(hit.speed, volume.0);
        if hit.channel == CH_SNARE {
            play_snare_web_audio(vol);
        } else if hit.channel >= CH_VIB_FIRST {
            play_vib_web_audio((hit.channel - CH_VIB_FIRST) as u32, vol);
        }
    }
}

#[cfg(target_arch = "wasm32")]
fn play_snare_web_audio(volume: f32) {
    AUDIO_CTX.with(|slot| {
        let borrow = slot.borrow();
        let Some(ctx) = borrow.as_ref() else { return };
        let rate = ctx.sample_rate();
        let n = (rate * 0.35) as u32;
        let Ok(buf) = ctx.create_buffer(1, n, rate) else { return };
        let samples = generate_snare_samples_f32(n as usize, rate as u32);
        let _ = buf.copy_to_channel(&samples, 0);
        let Ok(source) = ctx.create_buffer_source() else { return };
        source.set_buffer(Some(&buf));
        let Ok(gain) = ctx.create_gain() else { return };
        gain.gain().set_value(volume);
        let _ = source.connect_with_audio_node(&gain);
        let _ = gain.connect_with_audio_node(&ctx.destination());
        let _ = source.start();
    });
}

#[cfg(target_arch = "wasm32")]
fn play_vib_web_audio(bar_idx: u32, volume: f32) {
    AUDIO_CTX.with(|slot| {
        let borrow = slot.borrow();
        let Some(ctx) = borrow.as_ref() else { return };
        let rate = ctx.sample_rate();
        let n = (rate * 2.0) as u32;
        let Ok(buf) = ctx.create_buffer(1, n, rate) else { return };
        let samples = generate_vib_samples_f32(bar_idx, n as usize, rate as u32);
        let _ = buf.copy_to_channel(&samples, 0);
        let Ok(source) = ctx.create_buffer_source() else { return };
        source.set_buffer(Some(&buf));
        let Ok(gain) = ctx.create_gain() else { return };
        gain.gain().set_value(volume);
        let _ = source.connect_with_audio_node(&gain);
        let _ = gain.connect_with_audio_node(&ctx.destination());
        let _ = source.start();
    });
}

// ── Shared sample generation ──────────────────────────────────────────────────

fn generate_snare_samples_f32(n: usize, sample_rate: u32) -> Vec<f32> {
    let mut state: u32 = 0xDEAD_BEEF;
    (0..n)
        .map(|i| {
            let t = i as f32 / sample_rate as f32;
            state = state.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            let noise = (state >> 16) as f32 / 32768.0 - 1.0;
            let tone = (std::f32::consts::TAU * 180.0 * t).sin();
            let v = noise * (-25.0 * t).exp() * 0.75 + tone * (-12.0 * t).exp() * 0.25;
            v.clamp(-1.0, 1.0)
        })
        .collect()
}

fn vib_bar_freq(bar_idx: u32) -> f32 {
    174.61 * 2.0_f32.powf(bar_idx as f32 / 12.0)
}

fn generate_vib_samples_f32(bar_idx: u32, n: usize, sample_rate: u32) -> Vec<f32> {
    let freq = vib_bar_freq(bar_idx);
    (0..n)
        .map(|i| {
            let t = i as f32 / sample_rate as f32;
            let env = (-2.5 * t).exp();
            let tone = (std::f32::consts::TAU * freq * t).sin() * 0.80
                + (std::f32::consts::TAU * freq * 2.0 * t).sin() * 0.15
                + (std::f32::consts::TAU * freq * 3.0 * t).sin() * 0.05;
            (tone * env).clamp(-1.0, 1.0)
        })
        .collect()
}

#[cfg(not(target_arch = "wasm32"))]
fn generate_snare_wav() -> Vec<u8> {
    let sample_rate: u32 = 44100;
    let samples = generate_snare_samples_f32((sample_rate as f32 * 0.35) as usize, sample_rate);
    pcm_to_wav(samples, sample_rate)
}

#[cfg(not(target_arch = "wasm32"))]
fn generate_vib_wav(bar_idx: u32) -> Vec<u8> {
    let sample_rate: u32 = 44100;
    let samples =
        generate_vib_samples_f32(bar_idx, (sample_rate as f32 * 2.0) as usize, sample_rate);
    pcm_to_wav(samples, sample_rate)
}

#[cfg(not(target_arch = "wasm32"))]
fn pcm_to_wav(samples: Vec<f32>, sample_rate: u32) -> Vec<u8> {
    let data_len = (samples.len() * 2) as u32;
    let mut wav = Vec::with_capacity(44 + data_len as usize);
    wav.extend_from_slice(b"RIFF");
    wav.extend_from_slice(&(36 + data_len).to_le_bytes());
    wav.extend_from_slice(b"WAVE");
    wav.extend_from_slice(b"fmt ");
    wav.extend_from_slice(&16u32.to_le_bytes());
    wav.extend_from_slice(&1u16.to_le_bytes());
    wav.extend_from_slice(&1u16.to_le_bytes());
    wav.extend_from_slice(&sample_rate.to_le_bytes());
    wav.extend_from_slice(&(sample_rate * 2).to_le_bytes());
    wav.extend_from_slice(&2u16.to_le_bytes());
    wav.extend_from_slice(&16u16.to_le_bytes());
    wav.extend_from_slice(b"data");
    wav.extend_from_slice(&data_len.to_le_bytes());
    for s in samples {
        wav.extend_from_slice(&((s * 32767.0) as i16).to_le_bytes());
    }
    wav
}
