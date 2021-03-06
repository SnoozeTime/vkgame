use crate::ecs::systems::RenderingSystem;
use std::collections::{HashMap, HashSet};
use winit::{
    DeviceEvent, ElementState, Event, EventsLoop, KeyboardInput, ModifiersState, VirtualKeyCode,
    WindowEvent,
};

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
    Print,
    NextScene,
    PreviousScene,
}

/// Similar to winit modifier state. Didn't want to leak type from
/// other library just in case.
pub struct Modifiers {
    pub ctrl: bool,
    pub alt: bool,
}

impl From<ModifiersState> for Modifiers {
    fn from(modifier: ModifiersState) -> Modifiers {
        Modifiers {
            ctrl: modifier.ctrl,
            alt: modifier.alt,
        }
    }
}

impl Modifiers {
    fn new() -> Self {
        Modifiers {
            ctrl: false,
            alt: false,
        }
    }

    fn reset(&mut self) {
        self.ctrl = false;
        self.alt = false;
    }
}

// Again, same as winit.
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum MouseButton {
    Left,
    Middle,
    Right,
    Other(u8),
}

impl From<winit::MouseButton> for MouseButton {
    fn from(b: winit::MouseButton) -> MouseButton {
        match b {
            winit::MouseButton::Left => MouseButton::Left,
            winit::MouseButton::Right => MouseButton::Right,
            winit::MouseButton::Middle => MouseButton::Middle,
            winit::MouseButton::Other(u) => MouseButton::Other(u),
        }
    }
}

/// Abstract away winit input events.
/// Store the key pressed and so on.
///
/// Input is not really a system as it is inside the ECS and is passed to systems.
/// Also, there is no components for it.
///
/// TODO use backends. winit is for GUI backend but for server we need a different way to
/// do that.
pub struct Input {
    back: EventsLoop,

    mapping: HashMap<VirtualKeyCode, KeyType>,

    pub close_request: bool,
    resize_request: bool,

    is_window_focused: bool,

    // input state.
    axes: HashMap<Axis, f64>,
    keys: HashMap<KeyType, bool>,

    // mouse state.
    // TODO x, y instead of array, and don't put that public.
    pub mouse_pos: [f64; 2],
    buttons: HashSet<MouseButton>,

    // Maybe store one frame event. (e.g onKeyDown)
    keys_up: HashSet<KeyType>,
    keys_down: HashSet<KeyType>,
    pub modifiers: Modifiers,
}

impl Input {
    /// Rendering system is passed because it needs to pass the winit events
    /// to ImGUI.
    pub fn update(&mut self, rendering: &mut RenderingSystem) {
        // Reset events.
        self.modifiers.reset();
        self.buttons.clear();
        self.close_request = false;
        self.resize_request = false;
        self.keys_up.clear();
        self.keys_down.clear();
        self.axes.clear();

        let close_request = &mut self.close_request;
        let resize_request = &mut self.resize_request;
        let mapping = &self.mapping;
        let axes = &mut self.axes;
        let keys = &mut self.keys;
        let keys_up = &mut self.keys_up;
        let keys_down = &mut self.keys_down;
        let my_modifiers = &mut self.modifiers;
        let mouse_pos = &mut self.mouse_pos;
        let buttons = &mut self.buttons;
        let is_window_focused = &mut self.is_window_focused;

        // Now, poll keys.
        self.back.poll_events(|ev| {
            rendering.handle_event(&ev);

            if let Event::DeviceEvent { event, .. } = ev {
                if let DeviceEvent::MouseMotion { delta: (x, y) } = event {
                    // FOR AXIS
                    if *is_window_focused {
                        axes.insert(Axis::Horizontal, x);
                        axes.insert(Axis::Vertical, y);
                    }
                }
            } else if let Event::WindowEvent { event, .. } = ev {
                match event {
                    WindowEvent::Focused(focus) => *is_window_focused = focus,
                    WindowEvent::CloseRequested => *close_request = true,
                    WindowEvent::CursorMoved {
                        position,
                        modifiers,
                        ..
                    } => {
                        *my_modifiers = modifiers.into();
                        mouse_pos[0] = position.x;
                        mouse_pos[1] = position.y;
                    }
                    WindowEvent::Resized(_) => *resize_request = true,
                    WindowEvent::MouseInput {
                        button, modifiers, ..
                    } => {
                        buttons.insert(button.into());
                        *my_modifiers = modifiers.into();
                    }
                    WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                virtual_keycode: Some(keycode),
                                state,
                                modifiers,
                                ..
                            },
                        ..
                    } => {
                        *my_modifiers = modifiers.into();
                        if let Some(key) = mapping.get(&keycode) {
                            let key = (*key).clone();
                            match state {
                                ElementState::Pressed => {
                                    let old_state = *keys.get(&key).unwrap_or(&false);
                                    keys.insert(key.clone(), true);

                                    if !old_state {
                                        keys_down.insert(key);
                                    }
                                }
                                ElementState::Released => {
                                    let old_state = *keys.get(&key).unwrap_or(&false);
                                    keys.insert(key.clone(), false);

                                    if old_state {
                                        keys_up.insert(key);
                                    }
                                }
                            }
                        }
                    }
                    _ => (),
                }
            }
        });
    }

    pub fn new(back: EventsLoop) -> Self {
        let mut mapping = HashMap::new();
        mapping.insert(VirtualKeyCode::W, KeyType::Up);
        mapping.insert(VirtualKeyCode::S, KeyType::Down);
        mapping.insert(VirtualKeyCode::A, KeyType::Left);
        mapping.insert(VirtualKeyCode::D, KeyType::Right);
        mapping.insert(VirtualKeyCode::P, KeyType::Print);
        mapping.insert(VirtualKeyCode::Escape, KeyType::Escape);
        mapping.insert(VirtualKeyCode::Space, KeyType::Space);
        mapping.insert(VirtualKeyCode::Subtract, KeyType::PreviousScene);
        mapping.insert(VirtualKeyCode::Equals, KeyType::NextScene);

        Input {
            back,
            mapping,
            axes: HashMap::new(),
            keys: HashMap::new(),
            is_window_focused: true,

            mouse_pos: [0.0; 2],
            buttons: HashSet::new(),
            close_request: false,
            resize_request: false,
            keys_up: HashSet::new(),
            keys_down: HashSet::new(),
            modifiers: Modifiers::new(),
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

    pub fn get_mouse_clicked(&self, button: MouseButton) -> bool {
        self.buttons.contains(&button)
    }
}
