// Vertex shader

struct Camera {
    view_pos: vec4<f32>,
    view_proj: mat4x4<f32>,
}
@group(1) @binding(0)
var<uniform> camera: Camera;

struct Light {
    view_pos: vec4<f32>,
    view_proj: mat4x4<f32>,
    color: vec3<f32>,
    strength: f32,
}
@group(2) @binding(0)
var<uniform> light: Light;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) normal: vec3<f32>,
    @location(3) tangent: vec3<f32>,
    @location(4) bitangent: vec3<f32>,
}
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
    @location(0) tex_coords: vec2<f32>,
    @location(1) tangent_position: vec3<f32>,
    @location(2) tangent_light_position: vec3<f32>,
    @location(3) tangent_view_position: vec3<f32>,
    @location(4) world_position: vec4<f32>,
    @location(5) shadow_pos: vec3<f32>,
}

@vertex
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

    // Construct the tangent matrix
    let world_normal = normalize(normal_matrix * model.normal);
    let world_tangent = normalize(normal_matrix * model.tangent);
    let world_bitangent = normalize(normal_matrix * model.bitangent);
    let tangent_matrix = transpose(mat3x3<f32>(
        world_tangent,
        world_bitangent,
        world_normal,
    ));
    
    let world_position = model_matrix * vec4<f32>(model.position , 1.0);
    let pos_from_light: vec4<f32> = light.view_proj * world_position;
    var out: VertexOutput;
    out.world_position = world_position;
    out.clip_position = camera.view_proj * world_position;
    out.tex_coords = model.tex_coords;
    out.tangent_position = tangent_matrix * world_position.xyz;
    out.tangent_view_position = tangent_matrix * camera.view_pos.xyz;
    out.tangent_light_position = tangent_matrix * light.view_pos.xyz;
    out.shadow_pos = vec3<f32>(pos_from_light.xy * vec2<f32>(0.5, -0.5) + vec2<f32>(0.5, 0.5), pos_from_light.z);
    return out;
}


// Fragment shader
@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0)@binding(1)
var s_diffuse: sampler;
@group(0)@binding(2)
var t_normal: texture_2d<f32>;
@group(0) @binding(3)
var s_normal: sampler;
@group(0)@binding(4)
var t_specular: texture_2d<f32>;
@group(0) @binding(5)
var s_specular: sampler;

@group(3) @binding(0)
var t_shadow: texture_depth_2d;
@group(3) @binding(1)
var sampler_shadow: sampler_comparison;

// let acne_bias: f32 = 0.005;

fn fetch_shadow(homogeneous_coords: vec4<f32>) -> f32 {
    if (homogeneous_coords.w <= 0.0) {
        return 1.0;
    }
    // compensate for the Y-flip difference between the NDC and texture coordinates
    let flip_correction = vec2<f32>(0.5, -0.5);
    // compute texture coordinates for shadow lookup
    let proj_correction = 1.0 / homogeneous_coords.w;
    let light_local = homogeneous_coords.xy * flip_correction * proj_correction + vec2<f32>(0.5, 0.5);
    let depth_ref = (homogeneous_coords.z) * proj_correction;
    // do the lookup, using HW PCF and comparison
    return textureSampleCompareLevel(t_shadow, sampler_shadow, light_local, depth_ref);
}
// fn calc_shadow(shadow_pos: vec3<f32>) -> f32 {
//      // add shadow factor
//     var shadow : f32 = 0.0;
//     // apply Percentage-closer filtering (PCF)
//     // sample nearest 9 texels to smooth result
//     let size = f32(textureDimensions(t_shadow).x);
//     let range = 1;
//     for (var y : i32 = -range ; y <= range ; y = y + 1) {
//         for (var x : i32 = -range ; x <= range ; x = x + 1) {
//             let offset = vec2<f32>(f32(x) / size, f32(y) / size);
//             shadow = shadow + textureSampleCompareLevel(
//                 t_shadow,
//                 sampler_shadow,
//                 shadow_pos.xy ,
//                 shadow_pos.z - acne_bias  // apply a small bias to avoid acne
//             );
//         }
//     }
//     shadow = shadow / (f32(range + range + 1) * f32(range + range + 1));
//     return shadow;
// }

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let object_color: vec4<f32> = textureSample(t_diffuse, s_diffuse, in.tex_coords);
    let object_normal: vec4<f32> = textureSample(t_normal, s_normal, in.tex_coords);
    let object_specular: vec4<f32> = textureSample(t_specular, s_specular, in.tex_coords);


    let distance_from_light = distance(in.tangent_position, in.tangent_light_position);
    let reducer = distance_from_light;//(distance_from_light * distance_from_light);
    // We don't need (or want) much ambient light, so 0.1 is fine
    let ambient_strength = 0.1;
    let ambient_color = light.color * ambient_strength / distance_from_light;

    // Create the lighting vectors
    let tangent_normal = normalize(object_normal.xyz * 2.0 - 1.0);
    let light_dir = normalize(in.tangent_light_position - in.tangent_position);
    let view_dir = normalize(in.tangent_view_position - in.tangent_position);
    let half_dir = normalize(view_dir + light_dir);


    let diffuse_strength = clamp(dot(tangent_normal, light_dir), 0.0, 1.0) * light.strength;
    let diffuse_color = light.color * diffuse_strength / reducer;

    let specular_strength = pow(max(dot(tangent_normal, half_dir), 0.0), 32.0) * light.strength;
    let specular_color = specular_strength * light.color / reducer;


    let result = (ambient_color + diffuse_color) * object_color.xyz + specular_color * object_specular.xyz;

    let shadow = fetch_shadow(light.view_proj * in.world_position);
    // let shadow = calc_shadow(in.shadow_pos);
    let shadow = min(ambient_strength + shadow, 1.0);
    let result = shadow * result;
    return vec4<f32>(result, object_color.a);
} 