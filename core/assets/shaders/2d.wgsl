/////////////////////////////////////////////////

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec4<f32>,
    @location(2) texcoords: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) texcoords: vec2<f32>,
};

@vertex
fn vs_main(
    in: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.position = vec4<f32>(in.position, 1.0);
    out.color = in.color;
    out.texcoords = in.texcoords;
    return out;
}

/////////////////////////////////////////////////

@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0) @binding(1)
var s_diffuse: sampler;

@fragment
fn fs_main(
    in: VertexOutput
) -> @location(0) vec4<f32> {
    return in.color * textureSample(t_diffuse, s_diffuse, in.texcoords);
}
