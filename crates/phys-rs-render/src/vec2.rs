use std::ops::Add;

pub struct Vector2 {
    pub x: f32,
    pub y: f32,
}

impl Vector2 {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn zero() -> Self {
        Self { x: 0.0, y: 0.0 }
    }

    pub fn to_ndc(&self, window_size: (u32, u32)) -> Self {
        let x = self.x / window_size.0 as f32 * 2.0 - 1.0;
        let y = - (self.y / window_size.1 as f32 * 2.0 - 1.0);
        Self { x, y }
    }
}

impl From<Vector2> for [f32; 2] {
    fn from(vec: Vector2) -> Self {
        [vec.x, vec.y]
    }
}

impl Add<Vector2> for Vector2 {
    type Output = Vector2;

    fn add(self, other: Vector2) -> Vector2 {
        Vector2 {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}