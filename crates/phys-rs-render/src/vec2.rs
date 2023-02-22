use std::ops::{Add, Mul, AddAssign, Div, Sub, SubAssign, Not, Neg};

use lyon::geom::{euclid::{Point2D, UnknownUnit}, point};

#[derive(Debug, Clone, Copy, PartialEq)]
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

    pub fn one() -> Self {
        Self { x: 1.0, y: 1.0 }.normalize()
    }

    pub fn to_ndc(&self, window_size: (u32, u32)) -> Self {
        let x = self.x / window_size.0 as f32 * 2.0 - 1.0;
        let y = - (self.y / window_size.1 as f32 * 2.0 - 1.0);
        Self { x, y }
    }

    pub fn from_angle(angle: f32) -> Self {
        Self {
            x: angle.cos(),
            y: angle.sin(),
        }
    }

    pub fn normalize(&self) -> Self {
        let len = self.length();
        Self {
            x: self.x / len,
            y: self.y / len,
        }
    }

    pub fn length(&self) -> f32 {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    pub fn angle(&self) -> f32 {
        self.y.atan2(self.x)
    }

    pub fn rot_90cw(&self) -> Self {
        Self {
            x: self.y,
            y: -self.x,
        }
    }
}

impl From<Vector2> for [f32; 2] {
    fn from(vec: Vector2) -> Self {
        [vec.x, vec.y]
    }
}

impl From<Vector2> for Point2D<f32, UnknownUnit> {
    fn from(vec: Vector2) -> Self {
        point(vec.x, vec.y)
    }
}

impl From<[f32; 2]> for Vector2 {
    fn from(vec: [f32; 2]) -> Self {
        Self { x: vec[0], y: vec[1] }
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

impl Add<f32> for Vector2 {
    type Output = Vector2;

    fn add(self, other: f32) -> Vector2 {
        Vector2 {
            x: self.x + other,
            y: self.y + other,
        }
    }
}

impl AddAssign<Vector2> for Vector2 {
    fn add_assign(&mut self, other: Vector2) {
        self.x += other.x;
        self.y += other.y;
    }
}

impl Mul<f32> for Vector2 {
    type Output = Vector2;

    fn mul(self, other: f32) -> Vector2 {
        Vector2 {
            x: self.x * other,
            y: self.y * other,
        }
    }
}

impl Sub<Vector2> for Vector2 {
    type Output = Vector2;

    fn sub(self, other: Vector2) -> Vector2 {
        Vector2 {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

impl Sub<f32> for Vector2 {
    type Output = Vector2;

    fn sub(self, other: f32) -> Vector2 {
        Vector2 {
            x: self.x - other,
            y: self.y - other,
        }
    }
}

impl SubAssign<Vector2> for Vector2 {
    fn sub_assign(&mut self, other: Vector2) {
        self.x -= other.x;
        self.y -= other.y;
    }
}

impl Div<f32> for Vector2 {
    type Output = Vector2;

    fn div(self, other: f32) -> Vector2 {
        Vector2 {
            x: self.x / other,
            y: self.y / other,
        }
    }
}


impl Neg for Vector2 {
    type Output = Vector2;

    fn neg(self) -> Vector2 {
        Vector2 {
            x: -self.x,
            y: -self.y,
        }
    }
}