use twgraph::ecs::ComponentTemplate;
use twgraph::ecs::components::ModelComponent;
use serde_json;
fn main() {
    let mut c = ComponentTemplate{models: None, transforms: None};
    c.models = Some(ModelComponent{mesh_name:"ttt".to_string(),
    texture_name:"fd".to_string()});
    dbg!(serde_json::to_string(&c));
}
