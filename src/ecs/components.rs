use serde_derive::{Serialize, Deserialize};
use cgmath::Vector3;
use imgui::{Ui, im_str, ImGuiCond, ImGuiSelectableFlags, ImVec2};
use crate::editor::Editor;
use crate::ser::VectorDef;
use std::default::Default;


/// Hum that is just for the editor to have some human readable names.
/// Should be removed when packing the game.
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct NameComponent {
    pub name: String,
}

impl NameComponent {
    pub fn draw_ui(&mut self, _ui: &Ui, _editor: &Editor) {
        // nothing to see here.
    }
}

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

        if ui.small_button(im_str!("Select..")) {
            ui.open_popup(im_str!("select"));
        }
        ui.same_line(0.0);
        ui.text(im_str!("{}", self.mesh_name));
        ui.popup(im_str!("select"), || {  

            for model_name in &editor.all_models {
                let selected = *model_name == self.mesh_name;

                if ui.selectable(im_str!("{}", model_name), selected, 
                                 ImGuiSelectableFlags::empty(),
                                 ImVec2::new(0.0, 0.0)) {
                    self.mesh_name = (*model_name).clone();
                }
            }
        });

        if ui.small_button(im_str!("Select texture...")) {
            ui.open_popup(im_str!("select_texture"));
        }
        ui.same_line(0.0);
        ui.text(im_str!("{}", self.texture_name));
        ui.popup(im_str!("select_texture"), || {

            for texture_name in &editor.all_textures {
                let selected = *texture_name == self.texture_name;

                if ui.selectable(im_str!("{}", texture_name), selected, 
                                 ImGuiSelectableFlags::empty(),
                                 ImVec2::new(0.0, 0.0)) {
                    self.texture_name = (*texture_name).clone();
                }
            }

        });
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
    pub fn draw_ui(&mut self, ui: &Ui, _editor: &Editor) {
        ui.input_float(im_str!("speed"), &mut self.speed)
            .build();
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LightType {
    Point,
    Directional,
    Ambient,
}

// Emit light! Right now, only one is supported.
// An entity with a light component will need a transform.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LightComponent {
    // Should be between 0 and 1.0
    pub color: [f32; 3],
    pub light_type: LightType,
}

impl Default for LightComponent {

    fn default() -> Self {
        LightComponent {
            color: [1.0, 1.0, 1.0],
            light_type: LightType::Directional,
        }
    }
}

impl TransformComponent {
    pub fn draw_ui(&mut self, ui: &Ui, _editor: &Editor) {
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
    pub fn draw_ui(&mut self, ui: &Ui, editor: &mut Editor) {
        ui.input_float3(im_str!("color"), &mut self.color)
            .build();

        let selection = editor.components_state.get("light.type")
            .map(|s| s.clone()).unwrap_or(String::from("<None>"));
        // Then the variant.
        if ui.small_button(im_str!("Select..")) {
            ui.open_popup(im_str!("select"));
        }
        ui.same_line(0.0);

        ui.text(im_str!("{}", selection));

        ui.popup(im_str!("select"), || {  
            // SELECT POINT LIGHT
            if ui.selectable(
                im_str!("Point"),
                selection == "Point",
                ImGuiSelectableFlags::empty(),
                ImVec2::new(0.0, 0.0)) {

                editor.components_state.insert("light.type".to_string(), "Point".to_string());
                // Check if was a point light. If yes, do nothing. If no, transition.
                let new_type = match &self.light_type {
                    LightType::Point => None,
                    LightType::Directional => {
                        Some(LightType::Point)
                    },
                    LightType::Ambient => {
                        Some(LightType::Point)
                    }
                };


                if let Some(t) = new_type {
                    self.light_type = t;
                }
            }

            if ui.selectable(
                im_str!("Directional"),
                selection == "Directional",
                ImGuiSelectableFlags::empty(),
                ImVec2::new(0.0, 0.0)) {
                editor.components_state.insert("light.type".to_string(), "Directional".to_string());
                // Check if was a point light. If yes, do nothing. If no, transition.
                let new_type = match &self.light_type {
                    LightType::Point => {
                        Some(LightType::Directional)
                    },
                    LightType::Directional => {
                        None
                    },
                    LightType::Ambient => {
                        Some(LightType::Directional)
                    }
                };


                if let Some(t) = new_type {
                    self.light_type = t;
                }

            }


            if ui.selectable(
                im_str!("Ambient"),
                selection == "Ambient",
                ImGuiSelectableFlags::empty(),
                ImVec2::new(0.0, 0.0)) {

                editor.components_state.insert("light.type".to_string(), "Ambient".to_string());
                // Check if was a point light. If yes, do nothing. If no, transition.
                let new_type = match &self.light_type {
                    LightType::Point => {
                        Some(LightType::Ambient)
                    },
                    LightType::Directional => {
                        Some(LightType::Ambient)
                    },
                    LightType::Ambient => {
                        None 
                    }
                };


                if let Some(t) = new_type {
                    self.light_type = t;
                }

            }


        });

    }
}
