use std::{ffi::OsStr, path::Path};

use crate::{
    texture,
    utils::{self, Engine},
};

#[derive(Debug)]
pub struct Sky {
    pub pipeline: wgpu::RenderPipeline,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub texture: texture::Texture,
    pub bind_group: wgpu::BindGroup,
}
impl Sky {
    pub fn new<P>(p: P, engine: &Engine, camera_bind_group_layout: &wgpu::BindGroupLayout) -> Sky
    where
        P: AsRef<Path> + AsRef<OsStr>,
    {
        let bind_group_layout =
            engine
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("Skybox Texture bind group layout"),
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                                multisampled: false,
                                view_dimension: wgpu::TextureViewDimension::Cube,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                            count: None,
                        },
                    ],
                });
        let pipeline_layout =
            engine
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Skybox pipeline layout"),
                    bind_group_layouts: &[&camera_bind_group_layout, &bind_group_layout],
                    push_constant_ranges: &[],
                });
        let shader = wgpu::ShaderModuleDescriptor {
            label: Some("Normal Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/sky.wgsl").into()),
        };
        let pipeline = utils::create_skybox_pipeline(
            &engine.device,
            &pipeline_layout,
            engine.config.format,
            Some(texture::Texture::DEPTH_FORMAT),
            shader,
            Some("SKybox Render Pipeline"),
        );
        let sky_texture = {
            use std::fs::File;
            use std::io::{BufRead, BufReader};
            let f = File::open(p).unwrap();
            let mut reader = BufReader::new(f);
            let img_diffusion = ddsfile::Dds::read(&mut reader).unwrap();
            // let img_diffusion = image::io::Reader::open(p).unwrap().decode().unwrap();
            texture::Texture::from_cubemap_image(
                &engine.device,
                &engine.queue,
                &img_diffusion,
                Some("Skybox texture"),
            )
            .unwrap()
        };
        let bind_group = engine.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&sky_texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sky_texture.sampler),
                },
            ],
            label: None,
        });
        Sky {
            pipeline,
            bind_group_layout,
            texture: sky_texture,
            bind_group,
        }
    }
}
