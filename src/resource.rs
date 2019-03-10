use std::path::{PathBuf, Path};
use std::sync::Arc;
use vulkano::device::{Device, Queue};

use notify::{Watcher, RecommendedWatcher, RecursiveMode, watcher, DebouncedEvent};
use std::sync::mpsc::{Receiver, channel, TryRecvError};
use std::time::Duration;

use crate::event::{Event, ResourceEvent};
use crate::renderer::model::ModelManager;
use crate::renderer::texture::TextureManager;

pub struct Resources {
    pub models: ModelManager,
    pub textures: TextureManager,

    // Need to keep that in order to load new textures or models.
    device: Arc<Device>,
    queue: Arc<Queue>,

    // Will receive event from watcher.
    rx: Receiver<DebouncedEvent>,
    watcher: RecommendedWatcher,
    _resource_path: PathBuf,
}

impl Resources {

    pub fn new(device: Arc<Device>, queue: Arc<Queue>) -> Self {
        let textures = TextureManager::new();
        let models = ModelManager::new();

        // Create a channel to receive the events.
        let (tx, rx) = channel();
        let watcher = watcher(tx, Duration::from_secs(1)).unwrap();

        let resource_path = Path::new("assets");

        let mut r = Resources {
            models,
            textures,
            device,
            queue,
            rx,
            watcher,
            _resource_path: resource_path.to_path_buf(),
        };

        timed!(r.init_textures());
        timed!(r.init_models());


        // Create a watcher object, delivering debounced events.
        // The notification back-end is selected based on the platform.

        // Add a path to be watched. All files and directories at that path and
        // below will be monitored for changes.
        r.watcher.watch(resource_path, RecursiveMode::Recursive).unwrap();

        r
    }

    fn init_textures(&mut self) {
        self.textures.load_texture("bonjour".to_string(),
        Path::new("assets/image_img.png"),
        93, 93, self.device.clone(), self.queue.clone()).unwrap();
        self.textures.load_texture("white".to_string(), Path::new("assets/white.png"), 93, 93, self.device.clone(), self.queue.clone()).unwrap();
        self.textures.load_texture("red".to_string(), Path::new("assets/red.png"), 93, 93, self.device.clone(), self.queue.clone()).unwrap();
        self.textures.load_texture("blue".to_string(), Path::new("assets/blue.png"), 93, 93, self.device.clone(), self.queue.clone()).unwrap();
        self.textures.load_texture("green".to_string(), Path::new("assets/green.png"), 93, 93, self.device.clone(), self.queue.clone()).unwrap();
        //        self.textures.load_texture("floor".to_string(), Path::new("assets/textures/Concrete_Panels_001_COLOR.jpg"), 1024, 1024, self.device.clone(), self.queue.clone()).unwrap();
    }

    fn init_models(&mut self) {
        println!("Init models!");
        self.models.load_model("cube".to_string(), Path::new("assets/test1.obj"), self.device.clone()).expect("Cannot load model");
        self.models.load_model("floor".to_string(), Path::new("assets/floor.obj"), self.device.clone()).expect("Cannot load model");
        //self.models.load_model("building".to_string(), Path::new("assets/models/arena.obj"), self.device.clone()).expect("Cannot load model");

        println!("Finished reading models");
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
                        events.push(Event::ResourceEvent(ResourceEvent::ResourceReloaded(path)));
                    }
                },
                Err(TryRecvError::Empty) => break 'polling_loop,
                Err(TryRecvError::Disconnected) => panic!("Whhyyyy is that disconnected?"),
            }
        } 

        events
    }

}
