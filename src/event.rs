use crate::camera::CameraDirection;
use crate::scene::ClientCommand;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub enum Event {
    EditorEvent(EditorEvent),
    ResourceEvent(ResourceEvent),
    GameEvent(GameEvent),
    ClientEvent(ClientCommand),
}

/// Stuff that happens only in Editor.
#[derive(Debug, Clone)]
pub enum EditorEvent {
    PlayGame,
    ResourceEvent(ResourceEvent),
}

/// Events such as resource reloaded.
#[derive(Debug, Clone)]
pub enum ResourceEvent {
    ResourceReloaded(PathBuf),
}

#[derive(Debug, Clone)]
pub enum GameEvent {}
