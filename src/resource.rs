use log::*;
use notify::{watcher, DebouncedEvent, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Receiver, TryRecvError};
use std::sync::Arc;
use std::time::Duration;
use vulkano::device::Queue;

use crate::event::{Event, ResourceEvent};
use crate::renderer::model::ModelManager;
use crate::renderer::texture::TextureManager;

use std::ffi::OsStr;
use std::fs;

pub struct Resources {
    pub models: ModelManager,
    pub textures: TextureManager,

    // Need to keep that in order to load new textures or models.
    queue: Arc<Queue>,

    // Will receive event from watcher.
    rx: Receiver<DebouncedEvent>,
    watcher: RecommendedWatcher,
    resource_path: PathBuf,
}

impl Resources {
    pub fn new(queue: Arc<Queue>) -> Self {
        let textures = TextureManager::new();
        let models = ModelManager::new();

        // Create a channel to receive the events.
        let (tx, rx) = channel();
        let watcher = watcher(tx, Duration::from_secs(1)).unwrap();

        let resource_path = Path::new(option_env!("TWENGINE_ASSET_FOLDER").unwrap_or("assets"));

        let mut r = Resources {
            models,
            textures,
            queue,
            rx,
            watcher,
            resource_path: resource_path.to_path_buf(),
        };

        timed!(r.init_textures());
        timed!(r.init_models());

        // Create a watcher object, delivering debounced events.
        // The notification back-end is selected based on the platform.

        // Add a path to be watched. All files and directories at that path and
        // below will be monitored for changes.
        r.watcher
            .watch(resource_path, RecursiveMode::Recursive)
            .unwrap();

        r
    }

    /// List the current saved scene. This is used by the editor to easily switch between
    /// scenes.
    pub fn get_scene_names(&self) -> Vec<PathBuf> {
        let level_path = self.resource_path.join("levels");
        let mut paths = Vec::new();
        match std::fs::read_dir(&level_path) {
            Ok(readdir) => {
                for res in readdir {
                    match res {
                        Ok(dir_entry) => paths.push(dir_entry.path()),
                        Err(e) => error!("Error while reading DirEntry = {:?}", e),
                    }
                }
            }
            Err(e) => error!("Error while reading {:?} = {:?}", level_path, e),
        }

        paths
    }

    fn init_textures(&mut self) {
        // TODO Just use the path to load. String will be allocated automatically
        self.textures
            .load_texture(
                "white".to_string(),
                self.resource_path.join("white.png"),
                self.queue.device().clone(),
                self.queue.clone(),
            )
            .unwrap();
        self.textures
            .load_texture(
                "red".to_string(),
                self.resource_path.join("red.png"),
                self.queue.device().clone(),
                self.queue.clone(),
            )
            .unwrap();
        self.textures
            .load_texture(
                "blue".to_string(),
                self.resource_path.join("blue.png"),
                self.queue.device().clone(),
                self.queue.clone(),
            )
            .unwrap();
        self.textures
            .load_texture(
                "green".to_string(),
                self.resource_path.join("green.png"),
                self.queue.device().clone(),
                self.queue.clone(),
            )
            .unwrap();
        self.textures
            .load_texture(
                "green2".to_string(),
                self.resource_path.join("green2.png"),
                self.queue.device().clone(),
                self.queue.clone(),
            )
            .unwrap();
        self.textures
            .load_texture(
                "brown".to_string(),
                self.resource_path.join("brown.png"),
                self.queue.device().clone(),
                self.queue.clone(),
            )
            .unwrap();
        self.textures
            .load_texture(
                "tree1".to_string(),
                self.resource_path.join("tree1.png"),
                self.queue.device().clone(),
                self.queue.clone(),
            )
            .unwrap();
        self.textures
            .load_texture(
                "terrain1".to_string(),
                self.resource_path.join("terrain1.png"),
                self.queue.device().clone(),
                self.queue.clone(),
            )
            .unwrap();
    }

    fn init_models(&mut self) {
        debug!("Init models!");
        self.models
            .load_model(
                "cube".to_string(),
                self.resource_path.join("test1.obj"),
                self.queue.device().clone(),
            )
            .expect("Cannot load model");
        self.models
            .load_model(
                "floor".to_string(),
                self.resource_path.join("floor.obj"),
                self.queue.device().clone(),
            )
            .expect("Cannot load model");
        self.models
            .load_model(
                "room".to_string(),
                self.resource_path.join("room.obj"),
                self.queue.device().clone(),
            )
            .expect("Cannot load model");
        self.models
            .load_model(
                "tree1".to_string(),
                self.resource_path.join("tree1.obj"),
                self.queue.device().clone(),
            )
            .expect("Cannot load model");
        self.models
            .load_model(
                "terrain".to_string(),
                self.resource_path.join("terrain.obj"),
                self.queue.device().clone(),
            )
            .expect("cannot load terrain");

        debug!("Finished reading models");
    }

    fn reload_model(&mut self, path: &PathBuf) {
        if let Some(filename) = path.file_stem().and_then(|osstr| osstr.to_str()) {
            debug!("Will reload: {}", filename);
            if let Err(err) = self.models.load_model(
                filename.to_string(),
                path.to_path_buf(),
                self.queue.device().clone(),
            ) {
                debug!("Error while reloading model {:?}: {:?}", path, err);
            }
        }
    }

    fn reload_texture(&mut self, path: &PathBuf) {
        debug!("Reloading texture {:?}", path);
        if let Some(filename) = path
            .file_stem()
            .and_then(|osstr| osstr.to_str())
            .map(|s| s.to_string())
        {
            if let Err(err) = self.textures.load_texture(
                filename,
                (*path).clone(),
                self.queue.device().clone(),
                self.queue.clone(),
            ) {
                debug!("Error while reloading texture {:?}: {:?}", path, err);
            }
        }
    }

    /// Poll for resource events
    /// When a resource is updated, an event will be generated. Then, the relevant system
    /// can reload the resource.
    pub fn poll_events(&mut self) -> Vec<Event> {
        let mut events = Vec::new();

        'polling_loop: loop {
            let poll_result = self.rx.try_recv();
            match poll_result {
                Ok(ev) => {
                    if let DebouncedEvent::Write(path) = ev {
                        // Check if it is a model. If yes, reload/load it.
                        if let Some(extension) = path.extension() {
                            match extension {
                                x if x == OsStr::new("obj") => {
                                    self.reload_model(&path);
                                }
                                x if (x == OsStr::new("png"))
                                    || (x == OsStr::new("jpg"))
                                    || (x == OsStr::new("jpeg"))
                                    || (x == OsStr::new("JPEG"))
                                    || (x == OsStr::new("JPG"))
                                    || (x == OsStr::new("PNG")) =>
                                {
                                    self.reload_texture(&path);
                                }
                                _ => (),
                            }
                        }
                        events.push(Event::ResourceEvent(ResourceEvent::ResourceReloaded(path)));
                    }
                }
                Err(TryRecvError::Empty) => break 'polling_loop,
                Err(TryRecvError::Disconnected) => panic!("Whhyyyy is that disconnected?"),
            }
        }

        events
    }
}
