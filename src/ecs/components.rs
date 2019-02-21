use serde_derive::{Serialize, Deserialize};
use cgmath::Vector3;
use imgui::{Ui, im_str, ImGuiCond, ImGuiSelectableFlags, ImVec2};
use crate::editor::Editor;
use crate::ser::VectorDef;
use std::default::Default;

/// This is a component that is going to be rendered
/// by the render system.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ModelComponent {
    // name of the mesh and the texture, These need to be
    // loaded at the start of the scene.
    pub mesh_name: String,
    pub texture_name: String,
}

impl ModelComponent {
    pub fn draw_ui(&mut self, ui: &Ui, editor: &Editor) {
        ui.text(im_str!("Model: {}", self.mesh_name)); 
        ui.text(im_str!("Texture: {}", self.texture_name)); 
    }

}

impl Default for ModelComponent {
    fn default() -> Self {
        ModelComponent {
            mesh_name: "cube".to_string(),
            texture_name: "white".to_string(),
        }
    }
}

/// Position of the game object. No position = no rendering.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TransformComponent {
    #[serde(with = "VectorDef")]
    pub position: Vector3<f32>,

    #[serde(with = "VectorDef")]
    pub rotation: Vector3<f32>,

    #[serde(with = "VectorDef")]
    pub scale: Vector3<f32>,
}

impl Default for TransformComponent {
    
    fn default() -> Self {
        TransformComponent {
            position: Vector3::new(0.0, 0.0, 0.0),
            rotation: Vector3::new(0.0, 0.0, 0.0),
            scale: Vector3::new(1.0, 1.0, 1.0),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DummyComponent {
    pub speed: f32,
}

impl DummyComponent {
    pub fn draw_ui(&mut self, ui: &Ui, editor: &Editor) {
        ui.input_float(im_str!("speed"), &mut self.speed)
            .build();
    }
}

// Emit light! Right now, only one is supported.
// An entity with a light component will need a transform.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LightComponent {
    // Should be between 0 and 1.0
    pub color: [f32; 3],
}

impl TransformComponent {
    pub fn draw_ui(&mut self, ui: &Ui, editor: &Editor) {
        ui.tree_node(im_str!("position:")).opened(true, ImGuiCond::FirstUseEver).build(|| {
            ui.input_float(im_str!("x"), &mut self.position.x)
                .step(0.1)
                .step_fast(1.0)
                .build();
            ui.input_float(im_str!("y"), &mut self.position.y)
                .step(0.1)
                .step_fast(1.0)
                .build();
            ui.input_float(im_str!("z"), &mut self.position.z)
                .step(0.1)
                .step_fast(1.0)
                .build();
        });

        ui.tree_node(im_str!("rotation:")).opened(true, ImGuiCond::FirstUseEver).build(||{
            ui.input_float(im_str!("x"), &mut self.rotation.x)
                .step(0.1)
                .step_fast(1.0)
                .build();
            ui.input_float(im_str!("y"), &mut self.rotation.y)
                .step(0.1)
                .step_fast(1.0)
                .build();
            ui.input_float(im_str!("z"), &mut self.rotation.z)
                .step(0.1)
                .step_fast(1.0)
                .build();
        });

        ui.tree_node(im_str!("scale:")).opened(true, ImGuiCond::FirstUseEver).build(|| {
            ui.input_float(im_str!("x"), &mut self.scale.x)
                .step(0.1)
                .step_fast(1.0)
                .build();
            ui.input_float(im_str!("y"), &mut self.scale.y)
                .step(0.1)
                .step_fast(1.0)
                .build();
            ui.input_float(im_str!("z"), &mut self.scale.z)
                .step(0.1)
                .step_fast(1.0)
                .build();
        });
    }
}


impl LightComponent {
    pub fn draw_ui(&mut self, ui: &Ui, editor: &Editor) {
        ui.input_float3(im_str!("color"), &mut self.color)
            .build();
    }
}
