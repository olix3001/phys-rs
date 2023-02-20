struct Globals {
    u_resolution: vec2<f32>,
}

@group(0) @binding(0)
var<uniform> globals: Globals;

struct GridInput {
    @builtin(vertex_index) vertex_index: u32,
    @location(0) top_left: vec2<f32>,
    @location(1) bottom_right: vec2<f32>,
    @location(2) color: vec4<f32>,
    @location(3) spacing: f32,
    @location(4) thickness: f32,
    @location(5) subdivisions: u32,
}

struct GridOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) spacing: f32,
    @location(2) thickness: f32,
    @location(3) subdivisions: u32,
}

@vertex
fn vs_main(input: GridInput) -> GridOutput {
    var out: GridOutput;

    // Calculate the position of the vertex
    switch (input.vertex_index) {
        case 0u, 4u: { out.position = vec4<f32>(input.top_left.x, input.top_left.y, 0.0, 1.0); }
        case 3u: { out.position = vec4<f32>(input.bottom_right.x, input.top_left.y, 0.0, 1.0); }
        case 1u: { out.position = vec4<f32>(input.top_left.x, input.bottom_right.y, 0.0, 1.0); }
        case 2u, 5u: { out.position = vec4<f32>(input.bottom_right.x, input.bottom_right.y, 0.0, 1.0); }
        default: { out.position = vec4<f32>(0.0, 0.0, 0.0, 0.0); }
    }

    // Pass the input data to the fragment shader
    out.color = input.color;
    out.spacing = input.spacing;
    out.thickness = input.thickness;
    out.subdivisions = input.subdivisions;

    return out;
}


@fragment
fn fs_main(input: GridOutput) -> @location(0) vec4<f32> {
    // Big grid
    var uv = input.position.xy;
    if uv.x % input.spacing < input.thickness || uv.y % input.spacing < input.thickness {
        return input.color;
    }

    // Small grid
    if input.subdivisions > 0u {
        uv *= f32(input.subdivisions);
        if uv.x % input.spacing < input.thickness * 3.0 || uv.y % input.spacing < input.thickness * 3.0 {
            return vec4<f32>(input.color.xyz, input.color.w * 0.2);
        }
    }

    return vec4<f32>(0.0, 0.0, 0.0, 0.0);
}
