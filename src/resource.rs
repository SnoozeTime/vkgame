use std::path::Path;
use std::sync::Arc;
use vulkano::device::{Device, Queue};

use crate::renderer::model::ModelManager;
use crate::renderer::texture::TextureManager;

pub struct Resources {
    pub models: ModelManager,
    pub textures: TextureManager,

    // Need to keep that in order to load new textures or models.
    device: Arc<Device>,
    queue: Arc<Queue>,
}

impl Resources {

    pub fn new(device: Arc<Device>, queue: Arc<Queue>) -> Self {
        let textures = TextureManager::new();
        let models = ModelManager::new();

        let mut r = Resources {
            models,
            textures,
            device,
            queue,
        };

        r.init_textures();
        r.init_models();

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
        self.textures.load_texture("floor".to_string(), Path::new("assets/textures/Concrete_Panels_001_COLOR.jpg"), 1024, 1024, self.device.clone(), self.queue.clone()).unwrap();
    }

    fn init_models(&mut self) {
        println!("Init models!");
        self.models.load_model("cube".to_string(), Path::new("assets/test1.obj"), self.device.clone()).expect("Cannot load model");
        self.models.load_model("floor".to_string(), Path::new("assets/floor.obj"), self.device.clone()).expect("Cannot load model");
        self.models.load_model("building".to_string(), Path::new("assets/models/arena.obj"), self.device.clone()).expect("Cannot load model");

        println!("Finished reading models");
    }



}
