use serde_derive::{Serialize, Deserialize};
use cgmath::Vector3;
use std::fs;
use std::fs::{OpenOptions, File};
use std::io::{Write, Read};
use std::collections::HashMap;

pub mod gen_index;
pub mod components;
pub mod systems;

use self::components::{
    TransformComponent,
    ModelComponent,
    DummyComponent,
    LightComponent,
    NameComponent,
    LightType,
};
use self::gen_index::{GenerationalIndexAllocator, GenerationalIndexArray, GenerationalIndex};
use crate::camera::Camera;
use crate::error::TwResult;

pub type Entity = GenerationalIndex;
type EntityArray<T> = GenerationalIndexArray<T>;

#[derive(Serialize, Deserialize, Debug)]
pub struct ECS {

    allocator: GenerationalIndexAllocator,
    // For now, static camera. TODO add it as a component.
    //
    #[serde(skip)]
    pub camera: Camera,

    pub components: Components,

    #[serde(skip)]
    #[serde(default = "ECS::load_templates")]
    templates: HashMap<String, ComponentTemplate>,
}

impl ECS {

    pub fn new() -> Self {
        // ----------
        let camera = Camera::default();

        ECS {
            camera,
            allocator: GenerationalIndexAllocator::new(),
            components: Components::new(),
            templates: ECS::load_templates(),
        }
    }

    pub fn new_from_existing(ecs: &ECS) -> Self {
        let j = serde_json::to_string(ecs).unwrap();
        serde_json::from_str(&j).unwrap()
    }

    /// return the index of live entities.
    pub fn nb_entities(&self) -> Vec<Entity> {
        self.allocator.live_entities() 
    }

    pub fn load_templates() -> HashMap<String, ComponentTemplate> {
        let mut templates = HashMap::new();
        let paths = fs::read_dir("./templates/").unwrap()
            .map(|p| p.unwrap().path());

        for path in paths {
            // ew
            let name = {
                let name = path.file_stem().unwrap().to_string_lossy();
                name.to_string()
            };
            let mut file = File::open(path).unwrap();
            let mut content = String::new();
            file.read_to_string(&mut content).unwrap();
            let template: ComponentTemplate = serde_json::from_str(&content).unwrap();

            templates.insert(name.to_string(), template);
        }

        templates
    }

    pub fn new_entity(&mut self) -> GenerationalIndex {
        let index = self.allocator.allocate();
        self.components.new_entity(&index);
        index
    }

    pub fn delete_entity(&mut self, entity: &GenerationalIndex) {
        if !self.allocator.deallocate(*entity) {
            println!("Didn't deallocate");
        } else {
            println!("Correctly DESTROYED the entity");
        }
    }

    pub fn new_entity_from_template(&mut self, template_name: String) -> Option<GenerationalIndex> {

        let template = self.templates.get(&template_name);
        if let Some(template) = template {
            let index = self.allocator.allocate();
            let cloned = template.clone();
            self.components.new_from_template(&index, cloned);
            Some(index)
        } else {

            None
        }
    }

    pub fn dummy_ecs() -> ECS {

        let mut ecs = ECS::new();

        // First entity
        let id1 = ecs.new_entity();
        let id2 = ecs.new_entity();
        let id3 = ecs.new_entity();

        // tree
        let id4 = ecs.new_entity();

        let components = &mut ecs.components;
        components.transforms.set(&id1, TransformComponent {
            position: Vector3::new(0.0, 0.0, 1.0),
            rotation: Vector3::new(0.0, 0.0, 0.0),
            scale: Vector3::new(1.0, 1.0, 1.0),
        });
        components.models.set(&id1, ModelComponent {
            mesh_name: "room".to_owned(),
            texture_name: "red".to_owned(),
        });

        // Second entity
        components.transforms.set(&id2, TransformComponent {
            position: Vector3::new(0.0, 0.0, 0.0),
            rotation: Vector3::new(0.0, 0.0, 0.0),
            scale: Vector3::new(1.0, 1.0, 1.0),
        });
        components.models.set(&id2, ModelComponent {
            mesh_name: "floor".to_owned(),
            texture_name: "green".to_owned(),
        });

        components.transforms.set(&id3, TransformComponent {
            position: Vector3::new(1.0, 5.0, 1.0),
            rotation: Vector3::new(0.0, 0.0, 0.0),
            scale: Vector3::new(1.0, 1.0, 1.0),
        });
        components.lights.set(&id3, LightComponent {
            color: [1.0, 1.0, 1.0],
            light_type: LightType::Directional,
        });

        // My tree
        components.transforms.set(&id4, TransformComponent {
            position: Vector3::new(0.0, 0.0, 1.0),
            rotation: Vector3::new(0.0, 0.0, 0.0),
            scale: Vector3::new(1.0, 1.0, 1.0),
        });
        components.models.set(&id4, ModelComponent {
            mesh_name: "tree1".to_owned(),
            texture_name: "tree1".to_owned(),
        });

        ecs
    }

    pub fn save<P: AsRef<std::path::Path>>(&self, path: P) -> TwResult<()> {
        let mut file =  OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(path)?;

        let j = serde_json::to_string(self)?;
        write!(file, "{}", j)?;

        Ok(())
    }

    pub fn load<P: AsRef<std::path::Path>>(path: P) -> TwResult<Self> {
        let mut file = File::open(path)?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;
        let ecs = serde_json::from_str(&content).unwrap();
        Ok(ecs)
    }

    pub fn load_and_replace<P: AsRef<std::path::Path>>(&mut self,
                                                       path: P) -> TwResult<()> {
        let new_ecs = ECS::load(path)?;

        self.components = new_ecs.components;
        self.allocator = new_ecs.allocator;

        Ok(())
    }
}

/// Macro to set up the component arrays in the ECS. It should be used with
macro_rules! register_components {
    { $([$name:ident, $component:ty, $gui_name:expr],)+ } =>
    {

        #[derive(Debug, Clone, Serialize, Deserialize)]
        pub struct Components {
            /// Size of the arrays. They all should be the same.
            current_size: usize,

            /// Arrays can be accessed with the get methods.
            $(
                #[serde(default="GenerationalIndexArray::new")]
                pub $name: EntityArray<$component>,   
            )+
        }

        impl Components {
            pub fn new() -> Self {
                Components {
                    current_size: 0,
                    $(
                        $name: GenerationalIndexArray(Vec::new()),
                        )+
                }
            }

            /// We assume that this method is called by the ECS, so the index
            /// should be safe (meaning that either all the arrays contain
            /// the index, or the index is the next one when pushing elements
            /// to the arrays)...
            pub fn new_entity(&mut self, entity: &GenerationalIndex) {
                if entity.index() == self.current_size {
                    $(
                        self.$name.push(None);
                    )+
                        self.current_size += 1;
                } else if entity.index() < self.current_size {
                    $(
                        self.$name.empty(entity);
                    )+
                } else {
                    panic!("Tried to add an entity with index {}, but components arrays
                    only have elements up to {} entities", entity.index(), self.current_size);
                }


            }

            pub fn new_from_template(&mut self,
                                     entity: &GenerationalIndex,
                                     template: ComponentTemplate) {
                self.new_entity(entity);

                $(
                    if template.$name.is_some() {
                        self.$name.set(entity, template.$name.unwrap());
                    }
                )+

            }
        }

        #[derive(Clone, Debug, Serialize, Deserialize)]
        pub struct ComponentTemplate {
            $(
                #[serde(skip_serializing_if = "Option::is_none")]
                #[serde(default)]
                pub $name: Option<$component>,
                )+
        }

        impl ComponentTemplate {
            pub fn new() -> Self {
                ComponentTemplate {
                    $(
                        $name: None,
                        )+
                }
            }

        }

        use imgui::{Ui, im_str, ImGuiCond, ImGuiSelectableFlags, ImVec2};
        use crate::editor::Editor;
        impl Components {
            pub fn draw_ui(&mut self, ui: &Ui, editor: &mut Editor) {

                if let Some(entity) = editor.selected_entity {
                    $(
                        let mut should_delete = false;
                        if let Some($name) = self.$name.get_mut(&entity) {
                            ui.tree_node(im_str!($gui_name)).opened(true, ImGuiCond::FirstUseEver).build(|| {
                                ui.same_line(0.0);
                                if ui.small_button(im_str!("Delete")) {
                                    should_delete = true;
                                }
                                $name.draw_ui(&ui, editor);
                            });
                        }

                        if should_delete {
                            self.$name.empty(&entity);
                        }
                    )+

                        new_component_popup(ui, editor);
                }
            }
        }

        pub fn new_component_popup(ui: &Ui, editor: &mut Editor) {

            if ui.button(im_str!("Add component"),
            (0.0, 0.0)) {
                ui.open_popup(im_str!("Add component"));
            }
            ui.popup_modal(im_str!("Add component"))
                .build(|| {

                    $(
                        let selected  = if let Some(n) = &editor.new_component_name {
                            *n == $gui_name.to_string()
                        } else {
                            false
                        };


                        if ui.selectable(im_str!($gui_name), selected, ImGuiSelectableFlags::from_bits(1<<0).unwrap(), ImVec2::new(0.0, 0.0)) {
                            editor.new_component_name = Some(String::from($gui_name));
                        }
                    )+

                        editor.hovered = ui.want_capture_mouse();


                    if ui.button(im_str!("Add"), (0.0, 0.0)) {
                        editor.should_add_comp = true;                                                                           
                        ui.close_current_popup();
                    }


                    if ui.button(im_str!("Close"),
                    (0.0, 0.0)) {
                        editor.new_component_name = None;
                        ui.close_current_popup();
                    }
                });

        }


        impl ECS {

            pub fn add_new_component_by_name(&mut self, entity: &Option<Entity>, comp_name: &Option<String>) 
            {
                entity.as_ref().map(|e| {
                    $(
                        if let Some(n) = comp_name {
                            if *n == $gui_name.to_string() {
                                self.components.$name.set(e, <$component>::default());
                                return;
                            }
                        }
                    )+

                });
            }

        }

    }


}

register_components!(
    [transforms, TransformComponent, "Transform"],
    [models, ModelComponent, "Model"],
    [dummies, DummyComponent, "Dummy"],
    [lights, LightComponent, "Light"],
    [names, NameComponent, "Name"],
    );
