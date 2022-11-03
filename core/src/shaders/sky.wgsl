struct SkyOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec3<f32>,
    // @location(1) X: vec3<f32>,
};

struct Camera {
    view_pos: vec4<f32>,
    view_proj: mat4x4<f32>,
    view: mat4x4<f32>,
    proj_inv: mat4x4<f32>,
}
@group(0) @binding(0)
var<uniform> camera: Camera;

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> SkyOutput {
    // hacky way to draw a large triangle
    let tmp1 = i32(vertex_index) / 2;
    let tmp2 = i32(vertex_index) & 1;
    let pos = vec4<f32>(
        f32(tmp1) * 4.0 - 1.0,
        f32(tmp2) * 4.0 - 1.0,
        1.0,
        1.0
    );

    // transposition = inversion for this orthonormal matrix
    let inv_model_view = transpose(mat3x3<f32>(camera.view[0].xyz, camera.view[1].xyz, camera.view[2].xyz));
    let unprojected = camera.proj_inv * pos;

    var out: SkyOutput;
    out.uv = inv_model_view * unprojected.xyz;
    out.position = pos;
    // out.X = normalize(-camera.view_pos.xyz);
    return out;
}
@group(1) @binding(0)
var r_texture: texture_cube<f32>;
@group(1) @binding(1)
var r_sampler: sampler;
@fragment
fn fs_main(in: SkyOutput) -> @location(0) vec4<f32> {
    // return textureSample(r_texture, r_sampler, in.position.xyz);
    return textureSample(r_texture, r_sampler, in.uv);
}