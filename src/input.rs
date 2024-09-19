use std::collections::HashSet;

#[derive(Debug, Clone)]
pub struct InputState {
    pressed: HashSet<winit::keyboard::KeyCode>,
    cursor_pos: Option<Position2D>,
    cursor_change: Option<Position2D>,
}
impl InputState {
    pub fn new() -> Self {
        Self {
            pressed: HashSet::new(),
            cursor_pos: None,
            cursor_change: None,
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

    pub fn update_cursor(&mut self, position: Position2D) {
        let prev = self.cursor_pos;
        self.cursor_pos = Some(position);
        let Some(prev) = prev else {
            return;
        };
        let change = Position2D {
            x: position.x - prev.x,
            y: position.y - prev.y,
        };
        self.cursor_change = Some(change);
    }
    pub fn cursor_change(&self) -> Option<Position2D> {
        self.cursor_change
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
