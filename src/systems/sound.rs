use avian3d::prelude::*;
use bevy::prelude::*;

use crate::components::snare::SnareDrum;
use crate::systems::marble::Marble;

const MAX_IMPACT_SPEED: f32 = 4.0; // m/s — marble free-fall from 0.80 m spawn height

#[derive(Resource)]
pub struct SnareVolume(pub f32);

impl Default for SnareVolume {
    fn default() -> Self { SnareVolume(0.5) }
}

fn impact_volume(speed: f32, vol: f32) -> f32 {
    // Square-root curve: quieter hits stay audible, loud hits don't clip
    (speed / MAX_IMPACT_SPEED).clamp(0.0, 1.0).powf(0.5) * vol
}

// ── Native: pre-bake WAV into a Bevy AudioSource ─────────────────────────────

#[cfg(not(target_arch = "wasm32"))]
use std::sync::Arc;

#[cfg(not(target_arch = "wasm32"))]
use bevy::audio::Volume;

#[cfg(not(target_arch = "wasm32"))]
#[derive(Resource)]
pub struct SnareHitSound(pub Handle<AudioSource>);

#[cfg(not(target_arch = "wasm32"))]
pub fn setup_snare_sound(
    mut audio_sources: ResMut<Assets<AudioSource>>,
    mut commands: Commands,
) {
    let handle = audio_sources.add(AudioSource {
        bytes: Arc::from(generate_snare_wav()),
    });
    commands.insert_resource(SnareHitSound(handle));
}

#[cfg(not(target_arch = "wasm32"))]
pub fn snare_hit_sound_system(
    mut events: MessageReader<CollisionStart>,
    marbles: Query<&LinearVelocity, With<Marble>>,
    snares: Query<(), With<SnareDrum>>,
    sound: Option<Res<SnareHitSound>>,
    snare_volume: Res<SnareVolume>,
    mut commands: Commands,
) {
    let Some(sound) = sound else { return };
    for event in events.read() {
        let (e1, e2) = (event.collider1, event.collider2);
        let marble_entity = if marbles.contains(e1) && snares.contains(e2) {
            e1
        } else if marbles.contains(e2) && snares.contains(e1) {
            e2
        } else {
            continue;
        };

        let speed = marbles.get(marble_entity)
            .map(|v| v.0.length())
            .unwrap_or(0.0);

        commands.spawn((
            AudioPlayer(sound.0.clone()),
            PlaybackSettings {
                volume: Volume::Linear(impact_volume(speed, snare_volume.0)),
                ..PlaybackSettings::ONCE
            },
        ));
    }
}

// ── WASM: Web Audio API via a reused thread-local AudioContext ────────────────

#[cfg(target_arch = "wasm32")]
thread_local! {
    static AUDIO_CTX: std::cell::RefCell<Option<web_sys::AudioContext>> =
        std::cell::RefCell::new(None);
}

#[cfg(target_arch = "wasm32")]
pub fn setup_snare_sound() {
    AUDIO_CTX.with(|slot| {
        *slot.borrow_mut() = web_sys::AudioContext::new().ok();
    });
}

#[cfg(target_arch = "wasm32")]
pub fn snare_hit_sound_system(
    mut events: MessageReader<CollisionStart>,
    marbles: Query<&LinearVelocity, With<Marble>>,
    snares: Query<(), With<SnareDrum>>,
    snare_volume: Res<SnareVolume>,
) {
    for event in events.read() {
        let (e1, e2) = (event.collider1, event.collider2);
        let marble_entity = if marbles.contains(e1) && snares.contains(e2) {
            e1
        } else if marbles.contains(e2) && snares.contains(e1) {
            e2
        } else {
            continue;
        };

        let speed = marbles.get(marble_entity)
            .map(|v| v.0.length())
            .unwrap_or(0.0);

        play_snare_web_audio(impact_volume(speed, snare_volume.0));
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

// ── Shared: deterministic sample generation (no rand dependency) ──────────────

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

#[cfg(not(target_arch = "wasm32"))]
fn generate_snare_wav() -> Vec<u8> {
    let sample_rate: u32 = 44100;
    let samples =
        generate_snare_samples_f32((sample_rate as f32 * 0.35) as usize, sample_rate);
    let data_len = (samples.len() * 2) as u32;
    let mut wav = Vec::with_capacity(44 + data_len as usize);
    wav.extend_from_slice(b"RIFF");
    wav.extend_from_slice(&(36 + data_len).to_le_bytes());
    wav.extend_from_slice(b"WAVE");
    wav.extend_from_slice(b"fmt ");
    wav.extend_from_slice(&16u32.to_le_bytes());
    wav.extend_from_slice(&1u16.to_le_bytes()); // PCM
    wav.extend_from_slice(&1u16.to_le_bytes()); // mono
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
