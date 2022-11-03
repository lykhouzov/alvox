use super::light;
use crate::{
    instance,
    model::{self, Vertex},
    texture,
    utils::Engine,
};
#[derive(Debug)]
pub struct Shadow {
    pub texture: texture::Texture,
    pub pipeline: wgpu::RenderPipeline,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,
}
impl Shadow {
    pub fn new(light: &light::Light, engine: &Engine) -> Shadow {
        let texture =
            texture::Texture::create_shadow_texture(&engine.device, "Shadow Depth Texture");
        let pipeline = {
            let layout = engine
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Shadow Pipeline Layout"),
                    bind_group_layouts: &[&light.bind_group_layout],
                    push_constant_ranges: &[],
                });
            let shader = wgpu::ShaderModuleDescriptor {
                label: Some("Shadow Shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/shadow.wgsl").into()),
            };
            // core::state::create_render_pipeline(
            //     &engine.device,
            //     &layout,
            //     engine.config.format,
            //     Some(texture::Texture::DEPTH_FORMAT),
            //     &[model::ModelVertex::desc(), instance::InstanceRaw::desc()],
            //     shader,
            //     Some("Shadow Render Pipeline")
            // )
            crate::utils::create_shadow_render_pipeline(
                &engine.device,
                &layout,
                Some(texture::Texture::DEPTH_FORMAT),
                &[model::ModelVertex::desc(), instance::InstanceRaw::desc()],
                shader,
            )
        };
        // let shadow_bind_group =
        let bind_group_layout =
            engine
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                multisampled: false,
                                sample_type: wgpu::TextureSampleType::Depth,
                                view_dimension: wgpu::TextureViewDimension::D2,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Comparison),
                            count: None,
                        },
                    ],
                    label: Some("Shadow Bind Group Layout"),
                });
        let bind_group = engine.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&texture.sampler),
                },
            ],
            label: Some("Shadow Bind Group"),
        });
        Shadow {
            texture,
            pipeline,
            bind_group_layout,
            bind_group,
        }
    }
}
