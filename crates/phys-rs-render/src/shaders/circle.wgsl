struct Globals {
    u_resolution: vec2<f32>,
}

@group(0) @binding(0)
var<uniform> globals: Globals;

struct CircleInput {
    @builtin(vertex_index) vertex_index: u32,
    @location(0) center: vec2<f32>,
    @location(1) radius: f32,
    @location(2) color: vec4<f32>,
    @location(3) thickness: f32,
}

struct CircleOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) radius: f32,
    @location(2) thickness: f32,
    @location(3) center: vec2<f32>,
}

fn to_ndc(position: vec2<f32>) -> vec2<f32> {
    return vec2<f32>(position.x / globals.u_resolution.x * 2.0 - 1.0, 1.0 - position.y / globals.u_resolution.y * 2.0);
}

@vertex
fn vs_main(input: CircleInput) -> CircleOutput {
    var out: CircleOutput;

    var center = to_ndc(input.center);

    var top_left = center - vec2<f32>(input.radius*2.0 /globals.u_resolution.x, -input.radius*2.0 / globals.u_resolution.y);
    var bottom_right = center + vec2<f32>(input.radius*2.0 / globals.u_resolution.x, -input.radius*2.0 / globals.u_resolution.y);

    // Calculate the position of the vertex
    switch (input.vertex_index) {
        case 0u, 4u: { out.position = vec4<f32>(top_left.x, top_left.y, 0.0, 1.0); }
        case 3u: { out.position = vec4<f32>(bottom_right.x, top_left.y, 0.0, 1.0); }
        case 1u: { out.position = vec4<f32>(top_left.x, bottom_right.y, 0.0, 1.0); }
        case 2u, 5u: { out.position = vec4<f32>(bottom_right.x, bottom_right.y, 0.0, 1.0); }
        default: { out.position = vec4<f32>(0.0, 0.0, 0.0, 0.0); }
    }

    // Pass the input data to the fragment shader
    out.color = input.color;
    out.radius = input.radius;
    out.thickness = input.thickness;
    out.center = input.center;

    return out;
}

@fragment
fn fs_main(input: CircleOutput) -> @location(0) vec4<f32> {
    // distance from the center of the circle
    var dist = length(input.position.xy - input.center);

    // if thickness is 0, draw filled circle (with anti-aliasing)
    if (input.thickness == 0.0) {
        var alpha = 1.0 - smoothstep(input.radius - 1.0, input.radius, dist);
        return vec4<f32>(input.color.rgb, input.color.a * alpha);
    }

    // if thickness is > 0, draw inside circle outline (with anti-aliasing)
    var alpha = 1.0 - smoothstep(input.thickness - 1.0, input.thickness, abs((dist + input.thickness) - input.radius));
    return vec4<f32>(input.color.rgb, input.color.a * alpha);

}
