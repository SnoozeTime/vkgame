use std::path::{PathBuf, Path};
use std::sync::Arc;
use vulkano::device::{Device, Queue};

use notify::{Watcher, RecommendedWatcher, RecursiveMode, watcher, DebouncedEvent};
use std::sync::mpsc::{Receiver, channel, TryRecvError};
use std::time::Duration;

use crate::event::{Event, ResourceEvent};
use crate::renderer::model::ModelManager;
use crate::renderer::texture::TextureManager;

use std::ffi::OsStr;

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
        self.textures.load_texture("green2".to_string(), Path::new("assets/green2.png"), 93, 93, self.device.clone(), self.queue.clone()).unwrap();
        self.textures.load_texture("brown".to_string(), Path::new("assets/brown.png"), 92, 92, self.device.clone(), self.queue.clone()).unwrap();
        self.textures.load_texture("tree1".to_string(), Path::new("assets/tree1.png"), 512, 512, self.device.clone(), self.queue.clone()).unwrap();
    }

    fn init_models(&mut self) {
        println!("Init models!");
        self.models.load_model("cube".to_string(), Path::new("assets/test1.obj"), self.device.clone()).expect("Cannot load model");
        self.models.load_model("floor".to_string(), Path::new("assets/floor.obj"), self.device.clone()).expect("Cannot load model");
        self.models.load_model("room".to_string(), Path::new("assets/room.obj"), self.device.clone()).expect("Cannot load model");
        self.models.load_model("tree1".to_string(), Path::new("assets/tree1.obj"), self.device.clone()).expect("Cannot load model");
        //self.models.load_model("building".to_string(), Path::new("assets/models/arena.obj"), self.device.clone()).expect("Cannot load model");

        println!("Finished reading models");
    }

    fn reload_model(&mut self, path: &PathBuf) {
        if let Some(filename) = path.file_stem().and_then(|osstr| osstr.to_str()) {
            println!("Will reload: {}", filename);
            self.models.load_model(filename.to_string(), &path, self.device.clone());
        }
    }

    fn reload_texture(&mut self, path: &PathBuf) {
        println!("Reloading texture {:?}", path);
        if let Some(filename) = path.file_stem()
            .and_then(|osstr| osstr.to_str())
            .map(|s| s.to_string()) {

        if let Some(texture) = self.textures.textures.get(&filename) {
                let w = texture.width;
                let h = texture.height;
                if let Err(err) = self.textures.load_texture(filename, &path, w, h, self.device.clone(), self.queue.clone()) {
                    println!("Error while reloading texture {:?}: {:?}", path, err);
                }
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
                                },
                                x if (x == OsStr::new("png")) 
                                    || (x == OsStr::new("jpg"))
                                    || (x == OsStr::new("jpeg"))
                                    || (x == OsStr::new("JPEG"))
                                    || (x == OsStr::new("JPG"))
                                    || (x == OsStr::new("PNG")) => {
                                    self.reload_texture(&path);
                                },    
                                _ => ()
                            }
                        }
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
