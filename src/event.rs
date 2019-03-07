use std::path::PathBuf;

pub enum Event {
    EditorEvent(EditorEvent),
    ResourceEvent(ResourceEvent),
}

/// Stuff that happens only in Editor.
pub enum EditorEvent {
    PlayGame,
    ResourceEvent(ResourceEvent),
}

/// Events such as resource reloaded.
pub enum ResourceEvent {
    ResourceReloaded(PathBuf),
}
