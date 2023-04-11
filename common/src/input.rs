use winit::event::*;

#[derive(Debug, Clone, Copy)]
enum KeyAction {
    Pressed(VirtualKeyCode),
    Released(VirtualKeyCode),
}

#[derive(Debug, Clone, Copy)]
enum MouseAction {
    Pressed(MouseButton),
    Released(MouseButton),
}

/// This impl makes a few assumptions:
/// 1. The window size will never change.
/// 2. There will only be one window.
#[derive(Debug, Clone)]
pub struct Input {
    mouse_actions: Vec<MouseAction>,
    key_actions: Vec<KeyAction>,
    key_held: [bool; 255],
    mouse_held: [bool; 255],
    cursor_pos: Option<(f64, f64)>,
    cursor_pos_prev: Option<(f64, f64)>,
}
impl Input {
    pub fn new() -> Self {
        Self {
            mouse_actions: vec![],
            key_actions: vec![],
            key_held: [false; 255],
            mouse_held: [false; 255],
            cursor_pos: None,
            cursor_pos_prev: None,
        }
    }

    pub fn update<T>(&mut self, event: &Event<T>) -> bool {
        match event {
            Event::WindowEvent {
                window_id: _,
                event,
            } => {
                self.handle_window_event(event);
                false
            }
            Event::MainEventsCleared => true,
            _ => false,
        }
    }

    pub fn step(&mut self) {
        self.mouse_actions = vec![];
        self.key_actions = vec![];
        self.cursor_pos_prev = self.cursor_pos;
    }

    fn _handle_device_event(&mut self, _event: &DeviceEvent) {
        // Will need to handle window focus
        todo!();
    }

    fn handle_window_event(&mut self, event: &WindowEvent) {
        match event {
            WindowEvent::KeyboardInput { input, .. } => {
                if let Some(keycode) = input.virtual_keycode {
                    match input.state {
                        ElementState::Pressed => {
                            self.key_held[keycode as usize] = true;
                            self.key_actions.push(KeyAction::Pressed(keycode));
                        }
                        ElementState::Released => {
                            self.key_held[keycode as usize] = false;
                            self.key_actions.push(KeyAction::Released(keycode));
                        }
                    }
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.cursor_pos = Some((position.x, position.y));
            }
            // Modifiers?
            WindowEvent::MouseInput { state, button, .. } => match state {
                ElementState::Pressed => {
                    self.mouse_held[mb_to_index(*button)] = true;
                    self.mouse_actions.push(MouseAction::Pressed(*button));
                }
                ElementState::Released => {
                    self.mouse_held[mb_to_index(*button)] = false;
                    self.mouse_actions.push(MouseAction::Released(*button));
                }
            },
            _ => {}
        }
    }

    pub fn key_pressed(&self, key: VirtualKeyCode) -> bool {
        self.key_actions
            .iter()
            .find(|action| match **action {
                KeyAction::Pressed(action_key) => key == action_key,
                _ => false,
            })
            .is_some()
    }
    pub fn key_released(&self, key: VirtualKeyCode) -> bool {
        self.key_actions
            .iter()
            .find(|action| match **action {
                KeyAction::Released(action_key) => key == action_key,
                _ => false,
            })
            .is_some()
    }
    pub fn key_held(&self, key: VirtualKeyCode) -> bool {
        self.key_held[key as usize]
    }

    pub fn mb_pressed(&self, button: MouseButton) -> bool {
        self.mouse_actions
            .iter()
            .find(|action| match **action {
                MouseAction::Pressed(action_button) => button == action_button,
                _ => false,
            })
            .is_some()
    }
    pub fn mb_released(&self, button: MouseButton) -> bool {
        self.mouse_actions
            .iter()
            .find(|action| match **action {
                MouseAction::Released(action_button) => button == action_button,
                _ => false,
            })
            .is_some()
    }
    pub fn mb_held(&self, button: MouseButton) -> bool {
        self.mouse_held[mb_to_index(button)]
    }

    pub fn cursor_diff(&self) -> (f64, f64) {
        if let Some(c) = self.cursor_pos {
            if let Some(p) = self.cursor_pos_prev {
                return (c.0 - p.0, c.1 - p.1);
            }
        }
        (0.0, 0.0)
    }
    pub fn cursor_pos(&self) -> Option<(f64, f64)> {
        self.cursor_pos
    }
}

fn mb_to_index(button: MouseButton) -> usize {
    match button {
        MouseButton::Left => 0,
        MouseButton::Right => 1,
        MouseButton::Middle => 2,
        MouseButton::Other(i) => i as usize,
    }
}

