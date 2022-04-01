/////////////////////////////////////////////////
// Vertex shader

struct CameraUniform {
    view_pos: vec4<f32>;
    view_proj: mat4x4<f32>;
};
[[group(1), binding(0)]]
var<uniform> camera: CameraUniform;

struct Light {
    position: vec3<f32>;
    color: vec4<f32>;
};
let NUM_LIGHTS = 4;
struct LightBuffer {
    ambient_light: vec4<f32>;
    lights: array<Light, NUM_LIGHTS>;
};
[[group(2), binding(0)]]
var<uniform> light_buffer: LightBuffer;

struct InstanceInput {
    [[location(5)]] model_matrix_0: vec4<f32>;
    [[location(6)]] model_matrix_1: vec4<f32>;
    [[location(7)]] model_matrix_2: vec4<f32>;
    [[location(8)]] model_matrix_3: vec4<f32>;
    [[location(9)]] normal_matrix_0: vec3<f32>;
    [[location(10)]] normal_matrix_1: vec3<f32>;
    [[location(11)]] normal_matrix_2: vec3<f32>;
};

struct VertexInput {
    [[location(0)]] position: vec3<f32>;
    [[location(1)]] tex_coords: vec2<f32>;
    [[location(2)]] normal: vec3<f32>;
    [[location(3)]] tangent: vec3<f32>;
    [[location(4)]] bitangent: vec3<f32>;
};

struct VertexOutput {
    [[builtin(position)]] clip_position: vec4<f32>;
    [[location(0)]] tex_coords: vec2<f32>;
    [[location(1)]] tangent_position: vec3<f32>;
    [[location(2)]] tangent_view_position: vec3<f32>;
    [[location(3)]] tangent_light_position_0: vec3<f32>;
    [[location(4)]] tangent_light_position_1: vec3<f32>;
    [[location(5)]] tangent_light_position_2: vec3<f32>;
    [[location(6)]] tangent_light_position_3: vec3<f32>;
};

[[stage(vertex)]]
fn vs_main(
    model: VertexInput,
    instance: InstanceInput,
) -> VertexOutput {
    let model_matrix = mat4x4<f32>(
        instance.model_matrix_0,
        instance.model_matrix_1,
        instance.model_matrix_2,
        instance.model_matrix_3,
    );
    let normal_matrix = mat3x3<f32>(
        instance.normal_matrix_0,
        instance.normal_matrix_1,
        instance.normal_matrix_2,
    );

    let world_normal = normalize(normal_matrix * model.normal);
    let world_tangent = normalize(normal_matrix * model.tangent);
    let world_bitangent = normalize(normal_matrix * model.bitangent);
    let tangent_matrix = transpose(mat3x3<f32>(
        world_tangent,
        world_bitangent,
        world_normal,
    ));

    let world_position = model_matrix * vec4<f32>(model.position, 1.0);

    var out: VertexOutput;
    out.clip_position = camera.view_proj * world_position;
    out.tex_coords = model.tex_coords;
    out.tangent_position = tangent_matrix * world_position.xyz;
    out.tangent_view_position = tangent_matrix * camera.view_pos.xyz;

    out.tangent_light_position_0 = tangent_matrix * light_buffer.lights[0].position;
    out.tangent_light_position_1 = tangent_matrix * light_buffer.lights[1].position;
    out.tangent_light_position_2 = tangent_matrix * light_buffer.lights[2].position;
    out.tangent_light_position_3 = tangent_matrix * light_buffer.lights[3].position;

    return out;
}

/////////////////////////////////////////////////
// Fragment shader - basic Blinn-Phong shading w/ point lights

[[group(0), binding(0)]]
var t_diffuse: texture_2d<f32>;
[[group(0), binding(1)]]
var s_diffuse: sampler;
[[group(0), binding(2)]]
var t_normal: texture_2d<f32>;
[[group(0), binding(3)]]
var s_normal: sampler;

//FIXME
[[stage(fragment)]]
fn fs_main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    let object_color: vec4<f32> = textureSample(t_diffuse, s_diffuse, in.tex_coords);
    let object_normal: vec4<f32> = textureSample(t_normal, s_normal, in.tex_coords);

    var total_light = vec3<f32>(0.0, 0.0, 0.0);
    for (var i: i32 = 0; i < NUM_LIGHTS; i = i + 1) {
        var tangent_light_position: vec3<f32>;
        switch (i) {
            default: {break;}
            case 0: {tangent_light_position = in.tangent_light_position_0;}
            case 1: {tangent_light_position = in.tangent_light_position_1;}
            case 2: {tangent_light_position = in.tangent_light_position_2;}
            case 3: {tangent_light_position = in.tangent_light_position_3;}
        }

        let light = light_buffer.lights[i];
        let light_color = light.color.rgb * light.color.a;

        let tangent_normal = object_normal.xyz * 2.0 - 1.0; //?
        let light_dir = normalize(tangent_light_position - in.tangent_position);
        let view_dir = normalize(in.tangent_view_position - in.tangent_position);
        let half_dir = normalize(view_dir + light_dir);

        let diffuse_strength = max(dot(tangent_normal, light_dir), 0.0);
        let diffuse_color = light_color * diffuse_strength;

        let specular_strength = pow(max(dot(tangent_normal, half_dir), 0.0), 32.0);
        let specular_color = light_color * specular_strength;

        total_light = total_light + diffuse_color + specular_color;
    }

    total_light = total_light + (light_buffer.ambient_light.rgb * light_buffer.ambient_light.a);
    return vec4<f32>(object_color.rgb * total_light, object_color.a);
}