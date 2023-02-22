use crate::{Brush, math::Vector2, color::Color, ColorPalette, Renderer};

const DEFAULT_SPRING_WIDTH: f32 = 30.0;
const CONNECTOR_LENGTH: f32 = 15.0;
const BORDER_THICKNESS: f32 = 1.5;
const DEFAULT_ANGLE: f32 = 1.22; // about 70 degrees

pub fn draw_spring(brush: &mut Brush, renderer: &mut Renderer, a: Vector2, b: Vector2, k: f32, l0: f32, scale: f32) {
    let l = (a - b).length();
    let f = k * (l - l0);
    let dir = (b - a).normalize();
    let angle = -dir.angle();

    let x = l/l0;

    // Draw basic line
    brush.draw_line(a, b, 5.0, Color::from_hex(0xeeeeee10));
    
    // Calculate number of segments with length of l0 and default angle
    let l0a = l0 - CONNECTOR_LENGTH*scale*2.0;
    let segment_length = (DEFAULT_SPRING_WIDTH * scale / DEFAULT_ANGLE.sin()).abs();
    let segment_x_length = segment_length * DEFAULT_ANGLE.cos();
    let segment_count = (l0a / segment_x_length).floor() as u32;


    let la = l - CONNECTOR_LENGTH * scale *2.0;
    let rsl = la / segment_count as f32;
    let current_angle = (rsl / segment_length).acos();
    let current_segment_x_length = segment_length * current_angle.cos();
    let current_segment_y_length = segment_length * current_angle.sin();

    // Draw first segment (half length)
    let side_seg_trans = dir * (current_segment_x_length * 0.25) + dir.rot_90cw() * (current_segment_y_length * 0.25 - 2.0);
    brush._draw_quad_border_raw(a + dir * CONNECTOR_LENGTH * scale + side_seg_trans, 
        Vector2::new(segment_length * 0.5, 6.5 * scale),
        ColorPalette::WHITE, 
        BORDER_THICKNESS * scale,
        ColorPalette::BLACK,
        current_angle + angle, 0.0);

    // Draw segments

    for i in 0..segment_count-1 {
        let seg_pos = a + dir * CONNECTOR_LENGTH * scale + dir * current_segment_x_length * (i as f32) + dir * (current_segment_x_length);
        let seg_ang = current_angle * ((-1.0) + ((i%2) as f32) * 2.0);
        brush._draw_quad_border_raw(seg_pos, 
            Vector2::new(segment_length, 6.5 * scale),
            ColorPalette::WHITE, 
            BORDER_THICKNESS * scale,
            ColorPalette::BLACK,
            seg_ang + angle, 0.0);
    }

    // Draw last segment (half length)
    brush._draw_quad_border_raw(b - dir * CONNECTOR_LENGTH * scale - side_seg_trans, 
        Vector2::new(segment_length * 0.5, 6.5 * scale),
        ColorPalette::WHITE, 
        BORDER_THICKNESS * scale,
        ColorPalette::BLACK,
        current_angle + angle, 0.0);
    

    // Draw connector between first point and base
    brush.draw_line_rounded(a - dir * 3.0, a + dir * CONNECTOR_LENGTH * scale, 6.0, ColorPalette::WHITE);

    // Draw spring base 
    brush._draw_quad_border_raw(a + dir * CONNECTOR_LENGTH * scale, 
        Vector2::new(DEFAULT_SPRING_WIDTH * scale, 6.5 * scale),
        ColorPalette::WHITE, 
        BORDER_THICKNESS * scale,
        ColorPalette::BLACK,
        angle + 90f32.to_radians(), 0.0);

    // Draw connector between second point and base
    brush.draw_line_rounded(b + dir * 3.0, b - dir * CONNECTOR_LENGTH * scale, 6.0, ColorPalette::WHITE);

    // Draw spring base
    brush._draw_quad_border_raw(b - dir * CONNECTOR_LENGTH * scale, 
        Vector2::new(DEFAULT_SPRING_WIDTH * scale, 6.5 * scale),
        ColorPalette::WHITE, 
        BORDER_THICKNESS,
        ColorPalette::BLACK,
        angle + 90f32.to_radians(), 0.0);

    brush.flush(renderer);

    // Draw circles at connection points
    brush.draw_circle_filled(a, 2.5 * scale, ColorPalette::BLACK);
    brush.draw_circle_filled(b, 2.5 * scale, ColorPalette::BLACK);

}