use std::collections::HashSet;

#[derive(Debug, Clone)]
pub struct InputState {
    pressed: HashSet<winit::keyboard::KeyCode>,
}
impl InputState {
    pub fn new() -> Self {
        Self {
            pressed: HashSet::new(),
        }
    }

    pub fn update_key(&mut self, event: &winit::event::KeyEvent) {
        let winit::keyboard::PhysicalKey::Code(key) = event.physical_key else {
            return;
        };
        match event.state {
            winit::event::ElementState::Pressed => {
                self.pressed.insert(key);
            }
            winit::event::ElementState::Released => {
                self.pressed.remove(&key);
            }
        }
    }
    pub fn is_key_pressed(&self, key: winit::keyboard::KeyCode) -> bool {
        self.pressed.contains(&key)
    }
}
impl Default for InputState {
    fn default() -> Self {
        Self::new()
    }
}
