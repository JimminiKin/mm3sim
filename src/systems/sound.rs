use bevy::prelude::*;

use crate::resources::hihat_params::HiHatState;
use crate::systems::instrument::{
    InstrumentHits, CH_HIHAT, CH_KICK, CH_RIDE, CH_SNARE, CH_VIB_FIRST,
    CH_CAROUSEL_CRASH, CH_CAROUSEL_COWBELL, CH_CAROUSEL_TAMB, CH_CAROUSEL_WOOD,
};

const MAX_IMPACT_SPEED: f32 = 4.0;

#[cfg(not(target_arch = "wasm32"))]
use crate::resources::constants::VIB_BAR_COUNT;

#[derive(Resource)]
pub struct SnareVolume(pub f32);

impl Default for SnareVolume {
    fn default() -> Self {
        SnareVolume(0.5)
    }
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
#[derive(Resource)]
pub(crate) struct HiHatHitSounds {
    open: Handle<AudioSource>,
    closed: Handle<AudioSource>,
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Resource)]
pub(crate) struct KickHitSound(Handle<AudioSource>);

#[cfg(not(target_arch = "wasm32"))]
#[derive(Resource)]
pub(crate) struct RideHitSound(Handle<AudioSource>);

#[cfg(not(target_arch = "wasm32"))]
#[derive(Resource)]
pub(crate) struct CarouselHitSounds {
    pub crash: Handle<AudioSource>,
    pub cowbell: Handle<AudioSource>,
    pub tambourine: Handle<AudioSource>,
    pub woodblock: Handle<AudioSource>,
}

#[cfg(not(target_arch = "wasm32"))]
pub fn setup_sounds(mut audio_sources: ResMut<Assets<AudioSource>>, mut commands: Commands) {
    let snare = audio_sources.add(AudioSource {
        bytes: Arc::from(generate_snare_wav()),
    });
    commands.insert_resource(SnareHitSound(snare));

    let vib: Vec<_> = (0..VIB_BAR_COUNT)
        .map(|i| {
            audio_sources.add(AudioSource {
                bytes: Arc::from(generate_vib_wav(i)),
            })
        })
        .collect();
    commands.insert_resource(VibHitSounds(vib));

    let hihat_open = audio_sources.add(AudioSource {
        bytes: Arc::from(generate_hihat_wav(true)),
    });
    let hihat_closed = audio_sources.add(AudioSource {
        bytes: Arc::from(generate_hihat_wav(false)),
    });
    commands.insert_resource(HiHatHitSounds { open: hihat_open, closed: hihat_closed });

    let kick = audio_sources.add(AudioSource { bytes: Arc::from(generate_kick_wav()) });
    commands.insert_resource(KickHitSound(kick));

    let ride = audio_sources.add(AudioSource { bytes: Arc::from(generate_ride_wav()) });
    commands.insert_resource(RideHitSound(ride));

    let carousel = CarouselHitSounds {
        crash:      audio_sources.add(AudioSource { bytes: Arc::from(generate_crash_wav()) }),
        cowbell:    audio_sources.add(AudioSource { bytes: Arc::from(generate_cowbell_wav()) }),
        tambourine: audio_sources.add(AudioSource { bytes: Arc::from(generate_tambourine_wav()) }),
        woodblock:  audio_sources.add(AudioSource { bytes: Arc::from(generate_woodblock_wav()) }),
    };
    commands.insert_resource(carousel);
}

#[cfg(not(target_arch = "wasm32"))]
pub fn play_instrument_sounds(
    hits: Res<InstrumentHits>,
    snare_sound: Option<Res<SnareHitSound>>,
    vib_sounds: Option<Res<VibHitSounds>>,
    hihat_sounds: Option<Res<HiHatHitSounds>>,
    kick_sound: Option<Res<KickHitSound>>,
    ride_sound: Option<Res<RideHitSound>>,
    carousel_sounds: Option<Res<CarouselHitSounds>>,
    hihat_state: Res<HiHatState>,
    volume: Res<SnareVolume>,
    mut commands: Commands,
) {
    for hit in &hits.0 {
        let vol = impact_volume(hit.speed, volume.0);
        if hit.channel == CH_SNARE {
            let Some(s) = snare_sound.as_ref() else { continue };
            commands.spawn((AudioPlayer(s.0.clone()), PlaybackSettings { volume: Volume::Linear(vol), ..PlaybackSettings::ONCE }));
        } else if hit.channel == CH_HIHAT {
            let Some(sounds) = hihat_sounds.as_ref() else { continue };
            let handle = if hihat_state.open { &sounds.open } else { &sounds.closed };
            commands.spawn((AudioPlayer(handle.clone()), PlaybackSettings { volume: Volume::Linear(vol), ..PlaybackSettings::ONCE }));
        } else if hit.channel == CH_KICK {
            let Some(s) = kick_sound.as_ref() else { continue };
            commands.spawn((AudioPlayer(s.0.clone()), PlaybackSettings { volume: Volume::Linear(vol), ..PlaybackSettings::ONCE }));
        } else if hit.channel == CH_RIDE {
            let Some(s) = ride_sound.as_ref() else { continue };
            commands.spawn((AudioPlayer(s.0.clone()), PlaybackSettings { volume: Volume::Linear(vol), ..PlaybackSettings::ONCE }));
        } else if hit.channel == CH_CAROUSEL_CRASH
            || hit.channel == CH_CAROUSEL_COWBELL
            || hit.channel == CH_CAROUSEL_TAMB
            || hit.channel == CH_CAROUSEL_WOOD
        {
            let Some(sounds) = carousel_sounds.as_ref() else { continue };
            let handle = if hit.channel == CH_CAROUSEL_CRASH {
                &sounds.crash
            } else if hit.channel == CH_CAROUSEL_COWBELL {
                &sounds.cowbell
            } else if hit.channel == CH_CAROUSEL_TAMB {
                &sounds.tambourine
            } else {
                &sounds.woodblock
            };
            commands.spawn((AudioPlayer(handle.clone()), PlaybackSettings { volume: Volume::Linear(vol), ..PlaybackSettings::ONCE }));
        } else if hit.channel >= CH_VIB_FIRST {
            let Some(sounds) = vib_sounds.as_ref() else { continue };
            let bar_idx = hit.channel - CH_VIB_FIRST;
            let Some(handle) = sounds.0.get(bar_idx) else { continue };
            commands.spawn((AudioPlayer(handle.clone()), PlaybackSettings { volume: Volume::Linear(vol), ..PlaybackSettings::ONCE }));
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
    AUDIO_CTX.with(|slot| {
        *slot.borrow_mut() = web_sys::AudioContext::new().ok();
    });
}

#[cfg(target_arch = "wasm32")]
pub fn play_instrument_sounds(
    hits: Res<InstrumentHits>,
    volume: Res<SnareVolume>,
    hihat_state: Res<HiHatState>,
) {
    for hit in &hits.0 {
        let vol = impact_volume(hit.speed, volume.0);
        if hit.channel == CH_SNARE {
            play_snare_web_audio(vol);
        } else if hit.channel == CH_HIHAT {
            play_hihat_web_audio(vol, hihat_state.open);
        } else if hit.channel == CH_KICK {
            play_kick_web_audio(vol);
        } else if hit.channel == CH_RIDE {
            play_ride_web_audio(vol);
        } else if hit.channel == CH_CAROUSEL_CRASH {
            play_carousel_web_audio(vol, 0);
        } else if hit.channel == CH_CAROUSEL_COWBELL {
            play_carousel_web_audio(vol, 1);
        } else if hit.channel == CH_CAROUSEL_TAMB {
            play_carousel_web_audio(vol, 2);
        } else if hit.channel == CH_CAROUSEL_WOOD {
            play_carousel_web_audio(vol, 3);
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
        let Ok(buf) = ctx.create_buffer(1, n, rate) else {
            return;
        };
        let samples = generate_snare_samples_f32(n as usize, rate as u32);
        let _ = buf.copy_to_channel(&samples, 0);
        let Ok(source) = ctx.create_buffer_source() else {
            return;
        };
        source.set_buffer(Some(&buf));
        let Ok(gain) = ctx.create_gain() else { return };
        gain.gain().set_value(volume);
        let _ = source.connect_with_audio_node(&gain);
        let _ = gain.connect_with_audio_node(&ctx.destination());
        let _ = source.start();
    });
}

#[cfg(target_arch = "wasm32")]
fn play_hihat_web_audio(volume: f32, open: bool) {
    AUDIO_CTX.with(|slot| {
        let borrow = slot.borrow();
        let Some(ctx) = borrow.as_ref() else { return };
        let rate = ctx.sample_rate();
        let dur = if open { 0.40 } else { 0.12 };
        let n = (rate * dur) as u32;
        let Ok(buf) = ctx.create_buffer(1, n, rate) else { return };
        let samples = generate_hihat_samples_f32(n as usize, rate as u32, open);
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

#[cfg(target_arch = "wasm32")]
fn play_kick_web_audio(volume: f32) {
    AUDIO_CTX.with(|slot| {
        let borrow = slot.borrow();
        let Some(ctx) = borrow.as_ref() else { return };
        let rate = ctx.sample_rate();
        let n = (rate * 0.30) as u32;
        let Ok(buf) = ctx.create_buffer(1, n, rate) else { return };
        let samples = generate_kick_samples_f32(n as usize, rate as u32);
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
fn play_carousel_web_audio(volume: f32, slot: u8) {
    AUDIO_CTX.with(|cell| {
        let borrow = cell.borrow();
        let Some(ctx) = borrow.as_ref() else { return };
        let rate = ctx.sample_rate();
        let (dur, samples): (f32, Vec<f32>) = match slot {
            0 => (1.50, generate_crash_samples_f32((rate * 1.50) as usize, rate as u32)),
            1 => (0.40, generate_cowbell_samples_f32((rate * 0.40) as usize, rate as u32)),
            2 => (0.20, generate_tambourine_samples_f32((rate * 0.20) as usize, rate as u32)),
            _ => (0.10, generate_woodblock_samples_f32((rate * 0.10) as usize, rate as u32)),
        };
        let n = (rate * dur) as u32;
        let Ok(buf) = ctx.create_buffer(1, n, rate) else { return };
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
fn play_ride_web_audio(volume: f32) {
    AUDIO_CTX.with(|slot| {
        let borrow = slot.borrow();
        let Some(ctx) = borrow.as_ref() else { return };
        let rate = ctx.sample_rate();
        let n = (rate * 1.0) as u32;
        let Ok(buf) = ctx.create_buffer(1, n, rate) else { return };
        let samples = generate_ride_samples_f32(n as usize, rate as u32);
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

fn generate_kick_samples_f32(n: usize, sample_rate: u32) -> Vec<f32> {
    let mut state: u32 = 0xABCD_1234;
    (0..n)
        .map(|i| {
            let t = i as f32 / sample_rate as f32;
            state = state.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            let noise = (state >> 16) as f32 / 32768.0 - 1.0;
            // Pitch sweep: 100 Hz → 40 Hz punch
            let freq = 100.0 * (-15.0 * t).exp() + 40.0;
            let tone = (std::f32::consts::TAU * freq * t).sin();
            let env = (-8.0 * t).exp();
            // Sharp click transient on top
            let click = noise * (-80.0 * t).exp() * 0.30;
            ((tone * 0.70 + click) * env).clamp(-1.0, 1.0)
        })
        .collect()
}

fn generate_ride_samples_f32(n: usize, sample_rate: u32) -> Vec<f32> {
    // Long sustain, bright high-frequency shimmer
    let decay = 5.0_f32;
    let mut state: u32 = 0x9F8E_7D6C;
    (0..n)
        .map(|i| {
            let t = i as f32 / sample_rate as f32;
            state = state.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            let noise = (state >> 16) as f32 / 32768.0 - 1.0;
            // Higher partials than hi-hat → brighter character
            let ping = (std::f32::consts::TAU * 2000.0 * t).sin() * 0.08
                     + (std::f32::consts::TAU * 3200.0 * t).sin() * 0.06
                     + (std::f32::consts::TAU * 4500.0 * t).sin() * 0.04
                     + (std::f32::consts::TAU *  800.0 * t).sin() * 0.04;
            let env = (-decay * t).exp();
            ((noise * 0.60 + ping * 0.40) * env).clamp(-1.0, 1.0)
        })
        .collect()
}

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

fn generate_hihat_samples_f32(n: usize, sample_rate: u32, open: bool) -> Vec<f32> {
    // Open: slow decay → long shimmer; closed: fast decay → tight tick.
    let decay = if open { 15.0_f32 } else { 90.0_f32 };
    let mut state: u32 = 0xF1E2_D3C4;
    (0..n)
        .map(|i| {
            let t = i as f32 / sample_rate as f32;
            state = state.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            let noise = (state >> 16) as f32 / 32768.0 - 1.0;
            // Sparse metallic partials give the cymbal shimmer.
            let ping = (std::f32::consts::TAU * 800.0  * t).sin() * 0.06
                     + (std::f32::consts::TAU * 1500.0 * t).sin() * 0.04
                     + (std::f32::consts::TAU * 2400.0 * t).sin() * 0.02;
            let env = (-decay * t).exp();
            ((noise * 0.88 + ping * 0.12) * env).clamp(-1.0, 1.0)
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
fn generate_hihat_wav(open: bool) -> Vec<u8> {
    let sample_rate: u32 = 44100;
    let dur = if open { 0.40 } else { 0.12 };
    let samples = generate_hihat_samples_f32((sample_rate as f32 * dur) as usize, sample_rate, open);
    pcm_to_wav(samples, sample_rate)
}

#[cfg(not(target_arch = "wasm32"))]
fn generate_kick_wav() -> Vec<u8> {
    let sample_rate: u32 = 44100;
    let samples = generate_kick_samples_f32((sample_rate as f32 * 0.30) as usize, sample_rate);
    pcm_to_wav(samples, sample_rate)
}

#[cfg(not(target_arch = "wasm32"))]
fn generate_ride_wav() -> Vec<u8> {
    let sample_rate: u32 = 44100;
    let samples = generate_ride_samples_f32((sample_rate as f32 * 1.0) as usize, sample_rate);
    pcm_to_wav(samples, sample_rate)
}

fn generate_crash_samples_f32(n: usize, sample_rate: u32) -> Vec<f32> {
    // Wide-band noise with bright metallic shimmer and a long slow decay (~1.5 s).
    let decay = 3.0_f32;
    let mut state: u32 = 0xA1B2_C3D4;
    (0..n)
        .map(|i| {
            let t = i as f32 / sample_rate as f32;
            state = state.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            let noise = (state >> 16) as f32 / 32768.0 - 1.0;
            let shimmer = (std::f32::consts::TAU * 1800.0 * t).sin() * 0.05
                + (std::f32::consts::TAU * 3500.0 * t).sin() * 0.04
                + (std::f32::consts::TAU * 5200.0 * t).sin() * 0.03
                + (std::f32::consts::TAU * 7100.0 * t).sin() * 0.02;
            let env = (-decay * t).exp();
            ((noise * 0.86 + shimmer * 0.14) * env).clamp(-1.0, 1.0)
        })
        .collect()
}

fn generate_cowbell_samples_f32(n: usize, sample_rate: u32) -> Vec<f32> {
    // Classic cowbell: two slightly detuned metallic sine partials, medium decay.
    (0..n)
        .map(|i| {
            let t = i as f32 / sample_rate as f32;
            let env = (-12.0 * t).exp();
            let tone = (std::f32::consts::TAU * 562.0 * t).sin() * 0.55
                + (std::f32::consts::TAU * 845.0 * t).sin() * 0.28
                + (std::f32::consts::TAU * 1480.0 * t).sin() * 0.17;
            (tone * env).clamp(-1.0, 1.0)
        })
        .collect()
}

fn generate_tambourine_samples_f32(n: usize, sample_rate: u32) -> Vec<f32> {
    // Short jingle burst: high-frequency noise with ringing metallic partials.
    let decay = 30.0_f32;
    let mut state: u32 = 0x7788_9900;
    (0..n)
        .map(|i| {
            let t = i as f32 / sample_rate as f32;
            state = state.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            let noise = (state >> 16) as f32 / 32768.0 - 1.0;
            let jingle = (std::f32::consts::TAU * 3200.0 * t).sin() * 0.07
                + (std::f32::consts::TAU * 4800.0 * t).sin() * 0.05
                + (std::f32::consts::TAU * 6400.0 * t).sin() * 0.03;
            let env = (-decay * t).exp();
            ((noise * 0.85 + jingle * 0.15) * env).clamp(-1.0, 1.0)
        })
        .collect()
}

fn generate_woodblock_samples_f32(n: usize, sample_rate: u32) -> Vec<f32> {
    // Hollow knock: sharp transient click + 920 Hz resonant tone, very fast decay.
    let mut state: u32 = 0x1234_5678;
    (0..n)
        .map(|i| {
            let t = i as f32 / sample_rate as f32;
            state = state.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            let noise = (state >> 16) as f32 / 32768.0 - 1.0;
            let click = noise * (-120.0 * t).exp() * 0.35;
            let tone = (std::f32::consts::TAU * 920.0 * t).sin() * (-38.0 * t).exp() * 0.75;
            (click + tone).clamp(-1.0, 1.0)
        })
        .collect()
}

#[cfg(not(target_arch = "wasm32"))]
fn generate_crash_wav() -> Vec<u8> {
    let sample_rate: u32 = 44100;
    let samples = generate_crash_samples_f32((sample_rate as f32 * 1.5) as usize, sample_rate);
    pcm_to_wav(samples, sample_rate)
}

#[cfg(not(target_arch = "wasm32"))]
fn generate_cowbell_wav() -> Vec<u8> {
    let sample_rate: u32 = 44100;
    let samples = generate_cowbell_samples_f32((sample_rate as f32 * 0.40) as usize, sample_rate);
    pcm_to_wav(samples, sample_rate)
}

#[cfg(not(target_arch = "wasm32"))]
fn generate_tambourine_wav() -> Vec<u8> {
    let sample_rate: u32 = 44100;
    let samples = generate_tambourine_samples_f32((sample_rate as f32 * 0.20) as usize, sample_rate);
    pcm_to_wav(samples, sample_rate)
}

#[cfg(not(target_arch = "wasm32"))]
fn generate_woodblock_wav() -> Vec<u8> {
    let sample_rate: u32 = 44100;
    let samples = generate_woodblock_samples_f32((sample_rate as f32 * 0.10) as usize, sample_rate);
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
