use std::collections::HashSet;

#[derive(Debug, Clone)]
pub struct InputState {
    pressed: HashSet<winit::keyboard::KeyCode>,
    mouse_pos: Option<Position2D>,
    mouse_change: Option<Position2D>,
}
impl InputState {
    pub fn new() -> Self {
        Self {
            pressed: HashSet::new(),
            mouse_pos: None,
            mouse_change: None,
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

    pub fn update_mouse(&mut self, position: Position2D) {
        let prev = self.mouse_pos;
        self.mouse_pos = Some(position);
        let Some(prev) = prev else {
            return;
        };
        let change = Position2D {
            x: position.x - prev.x,
            y: position.y - prev.y,
        };
        self.mouse_change = Some(change);
    }
    pub fn mouse_change(&self) -> Option<Position2D> {
        self.mouse_change
    }
}
impl Default for InputState {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Position2D {
    pub x: f64,
    pub y: f64,
}
