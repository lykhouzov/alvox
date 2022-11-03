use cgmath::{InnerSpace, Vector2, Vector3};

use crate::chunk::Chunk;
use crate::{model, Position};

pub fn calc_normals(v: &[&model::ModelVertex]) -> ([[f32; 3]; 3], [[f32; 3]; 3], [[f32; 3]; 3]) {
    let ia_position = Vector3::from(v[0].position);
    let ib_position = Vector3::from(v[1].position);
    let ic_position = Vector3::from(v[2].position);

    let uv0: Vector2<_> = v[0].tex_coords.into();
    let uv1: Vector2<_> = v[1].tex_coords.into();
    let uv2: Vector2<_> = v[2].tex_coords.into();

    // This will give us a direction to calculate the
    // tangent and bitangent
    let delta_uv1 = uv1 - uv0;
    let delta_uv2 = uv2 - uv0;

    let e1 = ib_position - ia_position;
    let e2 = ic_position - ia_position;

    let no = Vector3::cross(e1, e2).normalize();

    let normal = [
        (Vector3::from(v[0].normal) + no).into(),
        (Vector3::from(v[1].normal) + no).into(),
        (Vector3::from(v[2].normal) + no).into(),
    ];
    // v[0].normal = (Vector3::from(v[0].normal) + no).into();
    // v[1].normal = (Vector3::from(v[1].normal) + no).into();
    // v[2].normal = (Vector3::from(v[2].normal) + no).into();

    // Solving the following system of equations will
    // give us the tangent and bitangent.
    //     delta_pos1 = delta_uv1.x * T + delta_u.y * B
    //     delta_pos2 = delta_uv2.x * T + delta_uv2.y * B
    // Luckily, the place I found this equation provided
    // the solution!
    let r = 1.0 / (delta_uv1.x * delta_uv2.y - delta_uv1.y * delta_uv2.x).max(0.00001);
    let tangent = (e1 * delta_uv2.y - e2 * delta_uv1.y) * r;

    let tangent = [
        (tangent + Vector3::from(v[0].tangent)).into(),
        (tangent + Vector3::from(v[1].tangent)).into(),
        (tangent + Vector3::from(v[2].tangent)).into(),
    ];
    // v[0].tangent = (tangent + Vector3::from(v[0].tangent)).into();
    // v[1].tangent = (tangent + Vector3::from(v[1].tangent)).into();
    // v[2].tangent = (tangent + Vector3::from(v[2].tangent)).into();
    // We flip the bitangent to enable right-handed normal
    // maps with wgpu texture coordinate system
    let bitangent = (e2 * delta_uv1.x - e1 * delta_uv2.x) * -r;

    let bitangent = [
        (bitangent + Vector3::from(v[0].bitangent)).into(),
        (bitangent + Vector3::from(v[1].bitangent)).into(),
        (bitangent + Vector3::from(v[2].bitangent)).into(),
    ];
    // v[0].bitangent = (bitangent + Vector3::from(v[0].bitangent)).into();
    // v[1].bitangent = (bitangent + Vector3::from(v[1].bitangent)).into();
    // v[2].bitangent = (bitangent + Vector3::from(v[2].bitangent)).into();
    (normal, tangent, bitangent)
}

#[allow(dead_code)]
pub fn to_index(position: &Position) -> usize {
    log::trace!("convert position {:?} to index", position);
    let Position { x, y, z } = position;
    let index = Chunk::WIDTH * Chunk::WIDTH * y.clone() as usize
        + Chunk::WIDTH * z.clone() as usize
        + x.clone() as usize;
    log::trace!("calculated index is {}", index);
    index
}

#[allow(dead_code)]
pub fn to_position(index: usize) -> Position {
    log::trace!("convert index {} to position", index);
    let x = index % Chunk::WIDTH;
    let y = ((index - x) / Chunk::WIDTH) % Chunk::WIDTH;
    let z = (index - x - Chunk::WIDTH * y) / (Chunk::WIDTH * Chunk::WIDTH);
    [x as f32, z as f32, y as f32].into()
}

pub async fn get_engine(window: &winit::window::Window) -> Engine {
    let size = window.inner_size();

    // The instance is a handle to our GPU
    // BackendBit::PRIMARY => Vulkan + Metal + DX12 + Browser WebGPU
    let instance = wgpu::Instance::new(wgpu::Backends::all());
    let surface = unsafe { instance.create_surface(window) };
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        })
        .await
        .unwrap();

    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                features: wgpu::Features::POLYGON_MODE_LINE
                    | wgpu::Features::DEPTH_CLIP_CONTROL
                    // | wgpu::Features::TEXTURE_COMPRESSION_ASTC_LDR
                    // | wgpu::Features::TEXTURE_COMPRESSION_ETC2
                    | wgpu::Features::TEXTURE_COMPRESSION_BC,
                limits: wgpu::Limits::default(),
            },
            None, // Trace path
        )
        .await
        .unwrap();

    let config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: surface
            .get_supported_formats(&adapter)
            .first()
            .unwrap()
            .to_owned(),
        width: size.width,
        height: size.height,
        present_mode: wgpu::PresentMode::Fifo,
    };
    surface.configure(&device, &config);
    Engine {
        device,
        queue,
        config,
        surface,
        scale_factor: window.scale_factor(),
    }
}
#[derive(Debug)]
pub struct Engine {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pub surface: wgpu::Surface,
    pub scale_factor: f64,
}

pub fn load_materials(
    engine: &Engine,
    bind_group_layout: &wgpu::BindGroupLayout,
) -> Vec<model::Material> {
    // /
    // / Load materials
    // /
    let out_dir = std::env::var("OUT_DIR").unwrap();
    println!("{:?}", &out_dir);
    let p = std::path::Path::new(&out_dir);
    // let root_path = &p.join("res").join("1k");
    let root_path = &p
        .join("res")
        .join("textures")
        .join("blocks")
        // .join("compressed")
        ;
    let materials =
        crate::texture::load_textures(&engine.device, &engine.queue, &bind_group_layout, root_path);
    materials
}

pub fn create_render_pipeline(
    device: &wgpu::Device,
    layout: &wgpu::PipelineLayout,
    color_format: wgpu::TextureFormat,
    depth_format: Option<wgpu::TextureFormat>,
    vertex_layouts: &[wgpu::VertexBufferLayout],
    shader: wgpu::ShaderModuleDescriptor,
    label: Option<&str>,
) -> wgpu::RenderPipeline {
    let shader = device.create_shader_module(shader);

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label,
        layout: Some(layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: "vs_main",
            buffers: vertex_layouts,
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: "fs_main",
            targets: &[Some(wgpu::ColorTargetState {
                format: color_format,
                blend: Some(wgpu::BlendState {
                    alpha: wgpu::BlendComponent::REPLACE,
                    color: wgpu::BlendComponent::REPLACE,
                }),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: Some(wgpu::Face::Back),
            // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
            polygon_mode: wgpu::PolygonMode::Fill,
            // Requires Features::DEPTH_CLIP_CONTROL
            unclipped_depth: false,
            // Requires Features::CONSERVATIVE_RASTERIZATION
            conservative: false,
        },
        depth_stencil: depth_format.map(|format| wgpu::DepthStencilState {
            format,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::Less,
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        }),
        multisample: wgpu::MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        multiview: None,
    })
}

pub fn create_skybox_pipeline(
    device: &wgpu::Device,
    layout: &wgpu::PipelineLayout,
    color_format: wgpu::TextureFormat,
    depth_format: Option<wgpu::TextureFormat>,
    shader: wgpu::ShaderModuleDescriptor,
    label: Option<&str>,
) -> wgpu::RenderPipeline {
    let shader = device.create_shader_module(shader);

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label,
        layout: Some(layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: "vs_main",
            buffers: &[],
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: "fs_main",
            targets: &[Some(wgpu::ColorTargetState {
                format: color_format,
                blend: Some(wgpu::BlendState {
                    alpha: wgpu::BlendComponent::REPLACE,
                    color: wgpu::BlendComponent::REPLACE,
                }),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        primitive: wgpu::PrimitiveState {
            front_face: wgpu::FrontFace::Cw,
            ..Default::default()
        },
        depth_stencil: depth_format.map(|format| wgpu::DepthStencilState {
            format,
            depth_write_enabled: false,
            depth_compare: wgpu::CompareFunction::LessEqual,
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        }),
        multisample: wgpu::MultisampleState::default(),
        multiview: None,
    })
}

pub fn create_shadow_render_pipeline(
    device: &wgpu::Device,
    layout: &wgpu::PipelineLayout,
    depth_format: Option<wgpu::TextureFormat>,
    vertex_layouts: &[wgpu::VertexBufferLayout],
    shader: wgpu::ShaderModuleDescriptor,
) -> wgpu::RenderPipeline {
    let shader = device.create_shader_module(shader);

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Shadow Render Pipeline"),
        layout: Some(layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: "vs_main",
            buffers: vertex_layouts,
        },
        fragment: None,
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: Some(wgpu::Face::Back),
            // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
            polygon_mode: wgpu::PolygonMode::Fill,
            // Requires Features::DEPTH_CLIP_CONTROL
            // unclipped_depth: false,
            // Requires Features::CONSERVATIVE_RASTERIZATION
            conservative: false,
            unclipped_depth: device
                .features()
                .contains(wgpu::Features::DEPTH_CLIP_CONTROL),
        },
        depth_stencil: depth_format.map(|format| wgpu::DepthStencilState {
            format,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::LessEqual,
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState {
                constant: 2, // corresponds to bilinear filtering
                slope_scale: 2.0,
                clamp: 0.0,
            },
        }),
        multisample: wgpu::MultisampleState::default(),
        multiview: None,
    })
}

pub fn create_output_buffer(texture_size: u32, device: &wgpu::Device) -> wgpu::Buffer {
    let u32_size = std::mem::size_of::<u32>() as u32;

    let output_buffer_size = (u32_size * texture_size * texture_size) as wgpu::BufferAddress;
    let output_buffer_desc = wgpu::BufferDescriptor {
        size: output_buffer_size,
        usage: wgpu::BufferUsages::COPY_DST
        // this tells wpgu that we want to read this buffer from the cpu
        | wgpu::BufferUsages::MAP_READ,
        label: None,
        mapped_at_creation: false,
    };
    device.create_buffer(&output_buffer_desc)
}
