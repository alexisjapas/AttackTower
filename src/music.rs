use bevy::audio::PlaybackMode;
use bevy::prelude::*;

use crate::common::*;

/// Background battle music, playing only while a match is live.
pub struct MusicPlugin;

impl Plugin for MusicPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_music)
            .add_systems(Update, sync_music_playback.in_set(AppSet::Visual));
    }
}

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
/// audio backend once the source is decoded) — when `Added<AudioSink>` fires
/// we re-read the current state, so a Menu→Playing transition that happens
/// before the sink exists still resolves correctly once it does.
pub fn sync_music_playback(
    state: Res<State<GameState>>,
    sinks: Query<&AudioSink, With<GameMusic>>,
    new_sinks: Query<Entity, (With<GameMusic>, Added<AudioSink>)>,
) {
    // `State<GameState>` is a resource mutated on every transition, so its
    // change detection still gates this system.
    if !state.is_changed() && new_sinks.is_empty() {
        return;
    }
    let should_play = *state.get() == GameState::Playing;
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
