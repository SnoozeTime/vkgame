use twgraph::ecs::ComponentTemplate;
use twgraph::ecs::components::{ModelComponent, TransformComponent, DummyComponent};

use cgmath::Vector3;
use std::fs::OpenOptions;
use std::env;

use std::io::Write;
fn main() {

    let mut args = env::args();

    let filename = if let Some(name) = args.nth(1) {
        name
    } else {
        "templates/template_test.json".to_owned()
    };
    
    let mut template = ComponentTemplate::new();
    template.models = Some(ModelComponent {
        mesh_name: "cube".to_string(),
        texture_name: "bonjour".to_string(),
    });

    template.transforms = Some(TransformComponent {
        position: Vector3::new(0.0, 1.0, 5.0),
        rotation: Vector3::new(0.0, 0.0, 0.0),
        scale: Vector3::new(1.0, 1.0, 1.0),
    });

    template.dummies = Some(DummyComponent {
        speed: 2.0,
    });

    let j = dbg!(serde_json::to_string(&template).unwrap());

    let mut file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(filename).unwrap();

    write!(file, "{}", j).unwrap();

}
