#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub const BLACK: Color = Color::new_const(0.0, 0.0, 0.0, 1.0);
    pub const WHITE: Color = Color::new_const(1.0, 1.0, 1.0, 1.0);

    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    pub const fn new_const(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    pub fn from_hex(hex: u32) -> Self {
        let r = ((hex >> 16) & 0xFF) as f32 / 255.0;
        let g = ((hex >> 8) & 0xFF) as f32 / 255.0;
        let b = (hex & 0xFF) as f32 / 255.0;
        let a = ((hex >> 24) & 0xFF) as f32 / 255.0;

        Self { r, g, b, a }
    }

    #[allow(clippy::wrong_self_convention)]
    pub fn to_linear_rgb(&self) -> [f32; 4] {
        let f = |xu: u32| {
            let x = (xu & 0xFF) as f32 / 255.0;
            if x > 0.04045 {
                ((x + 0.055) / 1.055).powf(2.4)
            } else {
                x / 12.92
            }
        };
        [
            f((self.r * 255.0) as u32),
            f((self.g * 255.0) as u32),
            f((self.b * 255.0) as u32),
            self.a]
    }
}

pub struct StandardColorPalette {}
impl StandardColorPalette {
    pub const BACKGROUND: Color = Color::new_const(0.0588, 0.0666, 0.0705, 1.0);
    pub const GRID: Color = Color::new_const(0.2, 0.2, 0.2, 1.0);

    pub const WHITE: Color = Color::new_const(1.0, 1.0, 250.0/255.0, 1.0);
    pub const BLUE: Color = Color::new_const(1.0/255.0, 111.0/255.0, 185.0/255.0, 1.0);
    pub const GREEN: Color = Color::new_const(4.0/255.0, 167.0/255.0, 119.0/255.0, 1.0);
    pub const RED: Color = Color::new_const(236.0/255.0, 78.0/255.0, 32.0/255.0, 1.0);
}


// ===< WGPU >===
impl From<Color> for wgpu::Color {
    fn from(color: Color) -> Self {
        let linear = color.to_linear_rgb();
        Self {
            r: linear[0] as f64,
            g: linear[1] as f64,
            b: linear[2] as f64,
            a: linear[3] as f64, 
        }
    }
}

impl From<Color> for [f32; 4] {
    fn from(color: Color) -> Self {
        color.to_linear_rgb()
    }
}