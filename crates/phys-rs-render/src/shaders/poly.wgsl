struct Globals {
    u_resolution: vec2<f32>,
}

struct Primitive {
    color: vec4<f32>,
    angle: f32,
    origin: vec2<f32>,

    count: u32,
}

@group(0) @binding(0)
var<uniform> globals: Globals;
@group(1) @binding(0)
var<uniform> u_primitives: array<Primitive, 256>;


struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) prim_index: u32,
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
}

fn to_ndc(position: vec2<f32>) -> vec2<f32> {
    return vec2<f32>(position.x / globals.u_resolution.x * 2.0 - 1.0, 1.0 - position.y / globals.u_resolution.y * 2.0);
}

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;

    // Rotate the point around the origin
    var angle = u_primitives[input.prim_index].angle;
    var origin = u_primitives[input.prim_index].origin;

    var rotated = vec2<f32>(
        (input.position.x - origin.x) * cos(angle) - (input.position.y - origin.y) * sin(angle) + origin.x,
        (input.position.x - origin.x) * sin(angle) + (input.position.y - origin.y) * cos(angle) + origin.y,
    );

    output.position = vec4<f32>(to_ndc(rotated), 0.0, 1.0);

    output.color = u_primitives[input.prim_index].color;

    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    return input.color;
}
