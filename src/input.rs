use std::collections::{HashSet, HashMap};
use winit::{KeyboardInput, VirtualKeyCode, EventsLoop, Event, WindowEvent, DeviceEvent, ElementState};

#[derive(Debug, Eq, PartialEq, Hash)]
pub enum Axis {
    Vertical,
    Horizontal,
}

#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub enum KeyType {
    Up,
    Down,
    Left,
    Right,
    Space,
    Escape,
}

/// Abstract away winit input events.
/// Store the key pressed and so on.
///
/// Input is not really a system as it is inside the ECS and is passed to systems.
/// Also, there is no components for it.
pub struct Input {
    back: EventsLoop,

    mapping: HashMap<VirtualKeyCode, KeyType>,


    pub close_request: bool,
    resize_request: bool,

    // input state.
    axes: HashMap<Axis, f64>,
    keys: HashMap<KeyType, bool>,

    // Maybe store one frame event. (e.g onKeyDown)
    keys_up: HashSet<KeyType>,
    keys_down: HashSet<KeyType>,
}

impl Input {

    pub fn update(&mut self) {

        // Reset events.
        self.close_request = false;
        self.resize_request = false;
        self.keys_up.clear();
        self.keys_down.clear();
        self.axes.clear();

        let mut close_request = &mut self.close_request;
        let mut resize_request = &mut self.resize_request;
        let mapping = &self.mapping;
        let axes = &mut self.axes;
        let keys = &mut self.keys;
        let keys_up = &mut self.keys_up;
        let keys_down = &mut self.keys_down;
        
        // Now, poll keys.
        self.back.poll_events(|ev| {

            if let Event::DeviceEvent { event, ..} = ev {
                if let DeviceEvent::MouseMotion { delta: (x, y) } = event {
                    // FOR AXIS
                    axes.insert(Axis::Horizontal, x);
                    axes.insert(Axis::Vertical, y);
                }
            } else if let Event::WindowEvent { event, ..} = ev {
                match event {
                    WindowEvent::CloseRequested => *close_request = true,
                    WindowEvent::Resized(_) => *resize_request = true,
                    WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                virtual_keycode: Some(keycode),
                                state,
                                ..
                            },
                            ..
                    } => {

                        if let Some(key) = mapping.get(&keycode) {
                            let key = (*key).clone();
                            match state {
                                ElementState::Pressed => {
                                    let old_state = *keys.get(&key).unwrap_or(&false);
                                    keys.insert(key.clone(), true);

                                    if !old_state {
                                        keys_down.insert(key);
                                    }
                                },
                                ElementState::Released => {
                                    let old_state = *keys.get(&key).unwrap_or(&false);
                                    keys.insert(key.clone(), false);

                                    if old_state {
                                        keys_up.insert(key);
                                    }
                                },
                            }
                        }
                    },
                            _ => (),
                }
            }});


    }

    pub fn new(back: EventsLoop) -> Self {
        let mut mapping = HashMap::new();
        mapping.insert(VirtualKeyCode::W, KeyType::Up);
        mapping.insert(VirtualKeyCode::S, KeyType::Down);
        mapping.insert(VirtualKeyCode::A, KeyType::Left);
        mapping.insert(VirtualKeyCode::D, KeyType::Right);
        mapping.insert(VirtualKeyCode::Escape, KeyType::Escape);

        Input {
            back,
            mapping,
            axes: HashMap::new(),
            keys: HashMap::new(),

            close_request: false,
            resize_request: false,
            keys_up: HashSet::new(),
            keys_down: HashSet::new(),
        }
    }

    pub fn get_axis(&self, axis: Axis) -> f64 {
        *self.axes.get(&axis).unwrap_or(&0.0)
    }

    pub fn get_key(&self, key: KeyType) -> bool {
        *self.keys.get(&key).unwrap_or(&false)
    }

    pub fn get_key_up(&self, key: KeyType) -> bool {
        self.keys_up.contains(&key)
    }

    pub fn get_key_down(&self, key: KeyType) -> bool {
        self.keys_down.contains(&key)
    }
}
