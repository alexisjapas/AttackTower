use bevy::audio::PlaybackMode;
use bevy::prelude::*;

use crate::common::*;

/// Marker on the looping music entity.
#[derive(Component)]
pub struct GameMusic;

/// Music asset path inside the `assets/` directory. The user is expected to
/// drop their file here under the matching name. If the file is missing the
/// loader logs a warning and the game continues to run silently.
pub const MUSIC_PATH: &str = "music/battleTheme.mp3";

pub fn setup_music(mut commands: Commands, asset_server: Res<AssetServer>) {
    let source: Handle<AudioSource> = asset_server.load(MUSIC_PATH);
    commands.spawn((
        AudioPlayer::<AudioSource>(source),
        PlaybackSettings {
            mode: PlaybackMode::Loop,
            paused: true,
            ..default()
        },
        GameMusic,
    ));
}

/// Keeps the music sink playing only while `GameState::Playing`. Pause, menu,
/// settings and end-of-game all pause the music. Reacts both to state changes
/// and to the moment the audio finally loads (`AudioSink` is inserted by the
/// audio backend once the source is decoded).
pub fn sync_music_playback(
    state: Res<GameState>,
    sinks: Query<&AudioSink, With<GameMusic>>,
    new_sinks: Query<Entity, (With<GameMusic>, Added<AudioSink>)>,
) {
    if !state.is_changed() && new_sinks.is_empty() {
        return;
    }
    let should_play = *state == GameState::Playing;
    for sink in &sinks {
        if should_play {
            if sink.is_paused() {
                sink.play();
            }
        } else if !sink.is_paused() {
            sink.pause();
        }
    }
}
