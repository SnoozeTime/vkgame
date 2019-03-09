use std::path::PathBuf;

#[derive(Debug, Clone)]
pub enum Event {
    EditorEvent(EditorEvent),
    ResourceEvent(ResourceEvent),
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
