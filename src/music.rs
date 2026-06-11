use bevy::audio::{PlaybackMode, Volume};
use bevy::prelude::*;

use crate::common::*;
use crate::graphics::music_volume_value;

/// Marker on the looping music entity.
#[derive(Component)]
pub struct GameMusic;

/// Music asset path inside the `assets/` directory. The user is expected to
/// drop their file here under the matching name. If the file is missing the
/// loader logs a warning and the game continues to run silently.
pub const MUSIC_PATH: &str = "music/battleTheme.mp3";

pub fn setup_music(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    settings: Res<GameSettings>,
) {
    let source: Handle<AudioSource> = asset_server.load(MUSIC_PATH);
    commands.spawn((
        AudioPlayer::<AudioSource>(source),
        PlaybackSettings {
            mode: PlaybackMode::Loop,
            paused: true,
            // Born at the persisted volume, so the async-created sink is right
            // even before `sync_music_playback` first touches it.
            volume: Volume::Linear(music_volume_value(settings.music_volume)),
            ..default()
        },
        GameMusic,
    ));
}

/// Keeps the music sink playing only while `GameState::Playing`. Pause, menu,
/// settings and end-of-game all pause the music. Reacts both to state changes
/// and to the moment the audio finally loads (`AudioSink` is inserted by the
/// audio backend once the source is decoded) — when `Added<AudioSink>` fires
/// we re-read the current state, so a Menu→Playing transition that happens
/// before the sink exists still resolves correctly once it does. Also applies
/// the `music_volume` setting whenever it changes.
pub fn sync_music_playback(
    state: Res<GameState>,
    settings: Res<GameSettings>,
    mut sinks: Query<&mut AudioSink, With<GameMusic>>,
    new_sinks: Query<Entity, (With<GameMusic>, Added<AudioSink>)>,
) {
    if !state.is_changed() && !settings.is_changed() && new_sinks.is_empty() {
        return;
    }
    let should_play = *state == GameState::Playing;
    let volume = Volume::Linear(music_volume_value(settings.music_volume));
    for mut sink in &mut sinks {
        if sink.volume() != volume {
            sink.set_volume(volume);
        }
        if should_play {
            if sink.is_paused() {
                sink.play();
            }
        } else if !sink.is_paused() {
            sink.pause();
        }
    }
}
