struct Globals {
    u_resolution: vec2<f32>,
}

@group(0) @binding(0)
var<uniform> globals: Globals;

struct VertexInput {
    @builtin(vertex_index) vertex_index: u32,
    @location(0) center: vec2<f32>,
    @location(1) size: vec2<f32>,
    @location(2) color: vec4<f32>,
    @location(3) thickness: f32,
    @location(4) border_radius: f32,
    @location(5) border_color: vec4<f32>,
    @location(6) angle: f32,
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) border_radius: f32,
    @location(2) center: vec2<f32>,
    @location(3) size: vec2<f32>,
    @location(4) border_color: vec4<f32>,
    @location(5) border_width: f32,
    @location(6) angle: f32,
}

fn screen_to_ndc(screen: vec2<f32>) -> vec2<f32> {
    // scale to ndc
    let ndc = screen * 2.0 / globals.u_resolution - 1.0;

    // flip y
    let ndc = vec2<f32>(ndc.x, -ndc.y);

    return ndc;
}

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var out: VertexOutput;

    var top_left = -input.size / 2.0;
    var bottom_right = input.size / 2.0;
    var top_right = vec2<f32>(bottom_right.x, top_left.y);
    var bottom_left = vec2<f32>(top_left.x, bottom_right.y);

    // Rotate point around center
    let rotation_matrix = mat2x2<f32>(
        cos(input.angle), -sin(input.angle),
        sin(input.angle), cos(input.angle)
    );

    top_left = rotation_matrix * top_left + input.center;
    bottom_right = rotation_matrix * bottom_right + input.center;
    top_right = rotation_matrix * top_right + input.center;
    bottom_left = rotation_matrix * bottom_left + input.center;

    // Draw the rectangle (CCW winding order)
    switch input.vertex_index {
        case 0u: { out.position = vec4<f32>(top_left, 0.0, 1.0); }
        case 1u: { out.position = vec4<f32>(bottom_left, 0.0, 1.0); }
        case 2u: { out.position = vec4<f32>(bottom_right, 0.0, 1.0); }
        case 3u: { out.position = vec4<f32>(top_right, 0.0, 1.0); }
        case 4u: { out.position = vec4<f32>(top_left, 0.0, 1.0); }
        case 5u: { out.position = vec4<f32>(bottom_right, 0.0, 1.0); }
        default: { out.position = vec4<f32>(0.0, 0.0, 0.0, 0.0); }
    }

    out.color = input.color;
    out.border_radius = input.border_radius;
    out.border_color = input.border_color;
    out.border_width = input.thickness;

    out.center = input.center;
    out.size = input.size;
    out.angle = input.angle;

    out.position = vec4<f32>(screen_to_ndc(out.position.xy), 0.0, 1.0);

    return out;
}


fn ndc_to_screen(ndc: vec2<f32>) -> vec2<f32> {
    // flip y
    let ndc = vec2<f32>(ndc.x, -ndc.y);

    // translate
    let ndc = ndc + vec2<f32>(1.0, 1.0);

    // scale to screen
    let screen = ndc * globals.u_resolution / 2.0;

    return screen;
}

fn box_dist(p: vec2<f32>, size: vec2<f32>, r: f32) -> f32 {
    let size = size - vec2<f32>(r, r);
    let d = abs(p) - size;
    return length(max(d, vec2<f32>(0.0))) + min(max(d.x, d.y), 0.0) - r;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    // Translate 

    let top_left = input.center - input.size / 2.0;
    let bottom_right = input.center + input.size / 2.0;

    let size = bottom_right - top_left;
    let center = top_left + size / 2.0;

    var color = input.color;

    // Move p relative to the center of the rectangle
    var p = input.position.xy - center;

    // Rotate the point back around the center
    let rotation_matrix = mat2x2<f32>(
        cos(-input.angle), -sin(-input.angle),
        sin(-input.angle), cos(-input.angle)
    );

    p = rotation_matrix * p;

    // calculate distance to the rectangle
    let dist = box_dist(p, size/2.0, input.border_radius);

    // Calculate the alpha
    let alpha = 1.0 - smoothstep(-0.75, -0.1, dist);

    // Draw the border if inner distance is less than the border width
    if (dist > -input.border_width && input.border_width > 0.0) {
        color = input.border_color;
    }

    // Return the color with the alpha
    return vec4<f32>(color.rgb, alpha * color.a);
}