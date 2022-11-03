// Vertex shader


struct Light {
    view_pos: vec4<f32>,
    view_proj: mat4x4<f32>,
    color: vec3<f32>,
    strength: f32,
}
@group(0) @binding(0)
var<uniform> light: Light;

struct VertexInput {
    @location(0) position: vec3<f32>,
};
struct InstanceInput {
    @location(5) model_matrix_0: vec4<f32>,
    @location(6) model_matrix_1: vec4<f32>,
    @location(7) model_matrix_2: vec4<f32>,
    @location(8) model_matrix_3: vec4<f32>,
    @location(9) normal_matrix_0: vec3<f32>,
    @location(10) normal_matrix_1: vec3<f32>,
    @location(11) normal_matrix_2: vec3<f32>,
}
struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
}

@vertex
fn vs_main(model: VertexInput, instance: InstanceInput) -> VertexOutput {
    let model_matrix = mat4x4<f32>(
        instance.model_matrix_0,
        instance.model_matrix_1,
        instance.model_matrix_2,
        instance.model_matrix_3,
    );

    let world_position = model_matrix * vec4<f32>(model.position, 1.0);
    let clip_position = light.view_proj * world_position;
    var out: VertexOutput;
    out.clip_position = clip_position;
    return out;
}


@fragment
fn fs_main(position: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(vec3<f32>(position.clip_position.z), 1.0);
}