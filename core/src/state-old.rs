use std::path::Path;
use std::{env, iter, vec};

use crate::light::{Light, LightController};
use crate::model::{DrawLight, DrawModel, Vertex};
use crate::texture::Texture;
use crate::utils::Engine;
use crate::{camera, instance, model, render, texture, utils, LightUniform};
use crate::{chunk::Chunk, CameraUniform};
use cgmath::Vector3;
use wgpu::util::DeviceExt;
use wgpu_glyph::{ab_glyph, GlyphBrushBuilder, Section, Text};
use winit::event::{ElementState, KeyboardInput, MouseButton, VirtualKeyCode};
use winit::{event::WindowEvent, window::Window};

#[allow(dead_code)]
pub(crate) struct State {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,
    render_pipeline: wgpu::RenderPipeline,
    // diffuse_bind_group: wgpu::BindGroup,
    camera: camera::Camera,
    camera_uniform: CameraUniform,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    pub camera_controller: camera::CameraController,
    projection: camera::Projection,
    pub mouse_pressed: bool,
    pub depth_texture: texture::Texture,
    pub glyph_brush: wgpu_glyph::GlyphBrush<wgpu::DepthStencilState>,
    pub fps: Vec<f32>,
    pub chunks: Vec<Chunk>,
    pub materials: Vec<model::Material>,
    light_uniform: crate::LightUniform,
    // light_buffer: wgpu::Buffer,
    // light_bind_group: wgpu::BindGroup,
    // light_render_pipeline: wgpu::RenderPipeline,
    pub light_model: Light,
    light_controller: LightController,
    light_render_pass: render::Pass,
    shadow_texture: Texture,
    light_projection: crate::light::Projection,
    shadow_render_pass: render::Pass,
    output_buffer: wgpu::Buffer,
}

impl State {
    pub async fn new(window: &Window) -> Self {
        let size = window.inner_size();

        let utils::Engine {
            device,
            queue,
            config,
            surface,
        } = utils::get_engine(window).await;
        // /
        // / TEXTURE
        // /
        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        // This should match the filterable field of the
                        // corresponding Texture entry above.
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 3,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 4,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 5,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
                label: Some("texture_bind_group_layout"),
            });

        // /
        // / Load materials
        // /
        let out_dir = env::var("OUT_DIR").unwrap();
        println!("{:?}", &out_dir);
        let p = Path::new(&out_dir);
        // let root_path = &p.join("res").join("1k");
        let root_path = &p.join("res").join("textures").join("blocks");
        let materials =
            texture::load_textures(&device, &queue, &texture_bind_group_layout, root_path);

        // /
        // / CAMERA
        // /
        let camera = camera::Camera::new((-1.0, 17.0, -1.0), cgmath::Deg(45.0), cgmath::Deg(-70.0));
        // let camera = camera::Camera::new((-22.5, 19.0, 6.0), cgmath::Deg(0.0), cgmath::Deg(-35.0));
        // let camera = camera::Camera::new((2.5, 10.0, 12.5), cgmath::Deg(-90.0), cgmath::Deg(-45.0));
        let projection =
            camera::Projection::new(config.width, config.height, cgmath::Deg(45.0), 0.1, 100.0);
        let camera_controller = camera::CameraController::new(4.0, 0.4);

        let mut camera_uniform = CameraUniform::new();
        camera_uniform.update_view_proj(&camera, &projection);

        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("camera_bind_group_layout"),
            });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
            label: Some("camera_bind_group"),
        });
        // /
        // / LIGHT
        // /
        let shadow_texture = Texture::create_shadow_texture(&device, "Shadow Texure");
        let light_model = Light::new((-1.0, 17.0, -1.0).into(), &device);
        let light_projection =
            crate::light::Projection::new(512, 512, cgmath::Deg(45.0), 0.1, 100.0);
        let mut light_uniform = LightUniform::new(200.0, [1.0, 1.0, 1.0]);
        light_uniform.update_view_proj(&light_model, &light_projection);
        let light_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Light VB"),
            contents: bytemuck::cast_slice(&[light_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let shadow_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Shadow Buffer"),
            contents: bytemuck::cast_slice(&[light_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let light_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            sample_type: wgpu::TextureSampleType::Depth,
                            view_dimension: wgpu::TextureViewDimension::D2,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Comparison),
                        count: None,
                    },
                ],
                label: None,
            });

        // /
        // / SHADOW
        // /
        let shadow_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: None,
            });
        let shadow_render_pipeline = {
            let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Shadow Pipeline Layout"),
                bind_group_layouts: &[&camera_bind_group_layout, &shadow_bind_group_layout],
                push_constant_ranges: &[],
            });
            let shader = wgpu::ShaderModuleDescriptor {
                label: Some("Shadow Shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("shaders/shadow.wgsl").into()),
            };
            create_shadow_render_pipeline(
                &device,
                &layout,
                Some(texture::Texture::DEPTH_FORMAT),
                &[model::ModelVertex::desc(), instance::InstanceRaw::desc()],
                shader,
            )
        };
        let shadow_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &shadow_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: shadow_buffer.as_entire_binding(),
            }],
            label: None,
        });
        let light_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &light_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: light_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&shadow_texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&shadow_texture.sampler),
                },
            ],
            label: None,
        });

        let light_render_pipeline = {
            let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Light Pipeline Layout"),
                bind_group_layouts: &[&camera_bind_group_layout, &light_bind_group_layout],
                push_constant_ranges: &[],
            });
            let shader = wgpu::ShaderModuleDescriptor {
                label: Some("Light Shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("shaders/light.wgsl").into()),
            };
            create_render_pipeline(
                &device,
                &layout,
                config.format,
                Some(texture::Texture::DEPTH_FORMAT),
                &[model::ModelVertex::desc()],
                shader,
                Some("Light Render Pipeline"),
            )
        };
        let light_render_pass = render::Pass {
            pipeline: light_render_pipeline,
            bind_group: light_bind_group,
            uniform_buf: light_buffer,
        };
        let shadow_render_pass = render::Pass {
            pipeline: shadow_render_pipeline,
            bind_group: shadow_bind_group,
            uniform_buf: shadow_buffer,
        };

        // /
        // / RENDER PIPELINE
        // /

        let render_pipeline = {
            let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[
                    &texture_bind_group_layout,
                    &camera_bind_group_layout,
                    &light_bind_group_layout,
                ],
                push_constant_ranges: &[],
            });
            let shader = wgpu::ShaderModuleDescriptor {
                label: Some("Normal Shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("shaders/shader.wgsl").into()),
            };
            create_render_pipeline(
                &device,
                &layout,
                config.format,
                Some(texture::Texture::DEPTH_FORMAT),
                &[model::ModelVertex::desc(), instance::InstanceRaw::desc()],
                shader,
                Some("Color Render Pipeline"),
            )
        };

        let mut chunks = vec![];
        for x in 0..1 {
            for z in 0..1 {
                let position =
                    Vector3::new((x * Chunk::WIDTH) as f32, 0.0, (z * Chunk::WIDTH) as f32);
                let chunk = Chunk::new(position, &device);
                chunks.push(chunk);
            }
        }

        // /
        // / DEPTH TEXTURE
        // /
        let depth_texture =
            texture::Texture::create_depth_texture(&device, &config, "depth_texture");
        // /
        // / FONT
        // /
        let font = ab_glyph::FontArc::try_from_slice(include_bytes!(
            "../res/Roboto_Mono/static/RobotoMono-Regular.ttf"
        ))
        .unwrap();
        let glyph_brush = GlyphBrushBuilder::using_font(font)
            .depth_stencil_state(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Greater,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            })
            .build(&device, wgpu::TextureFormat::Bgra8UnormSrgb);
        let light_controller = LightController::new(100.04, 0.01);

        let output_buffer = create_output_buffer(512, &device);
        Self {
            surface,
            device,
            queue,
            config,
            size,
            render_pipeline,
            // light_render_pipeline,
            camera,
            camera_uniform,
            camera_buffer,
            camera_bind_group,
            camera_controller,
            light_uniform,
            // light_buffer,
            // light_bind_group,
            light_controller,
            light_render_pass,
            projection,
            mouse_pressed: false,
            depth_texture,
            glyph_brush,
            fps: Vec::new(),
            chunks,
            light_model,
            materials,
            shadow_texture,
            light_projection,
            shadow_render_pass,
            output_buffer,
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.projection.resize(new_size.width, new_size.height);
            self.surface.configure(&self.device, &self.config);
            self.depth_texture =
                texture::Texture::create_depth_texture(&self.device, &self.config, "depth_texture");
        }
    }

    #[allow(unused_variables)]
    pub fn input(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        virtual_keycode: Some(key),
                        state,
                        ..
                    },
                ..
            } => {
                let r = self.camera_controller.process_keyboard(*key, *state);
                let r = self.light_controller.process_keyboard(*key, *state) && r;
                {
                    if state == &ElementState::Pressed && key == &VirtualKeyCode::F12 {
                        let buffer_slice = self.output_buffer.slice(..);
                        let texture_size = 512;

                        // NOTE: We have to create the mapping THEN device.poll() before await
                        // the future. Otherwise the application will freeze.
                        let (tx, rx) = futures_intrusive::channel::shared::oneshot_channel();
                        buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
                            tx.send(result).unwrap();
                        });
                        self.device.poll(wgpu::Maintain::Wait);
                        pollster::block_on(async { rx.receive().await.unwrap().unwrap() });

                        let data = buffer_slice.get_mapped_range();

                        use image::{ImageBuffer, Rgba};
                        let buffer =
                            ImageBuffer::<Rgba<u8>, _>::from_raw(texture_size, texture_size, data)
                                .unwrap();
                        buffer.save("image.png").unwrap();
                    };
                    if state == &ElementState::Released && key == &VirtualKeyCode::F12 {
                        self.output_buffer.unmap();
                    }
                }

                r
            }
            WindowEvent::MouseWheel { delta, .. } => {
                self.camera_controller.process_scroll(delta);
                true
            }
            WindowEvent::MouseInput {
                button: MouseButton::Left,
                state,
                ..
            } => {
                self.mouse_pressed = *state == ElementState::Pressed;
                true
            }
            _ => false,
        }
    }

    pub fn update(&mut self, dt: std::time::Duration) {
        self.camera_controller.update_camera(&mut self.camera, dt);
        self.camera_uniform
            .update_view_proj(&self.camera, &self.projection);
        self.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[self.camera_uniform]),
        );
        if self.fps.len() > 120 {
            self.fps = self.fps.drain(0..120).collect();
        }
        self.fps.push(dt.as_secs_f32());
        // Update the light
        self.light_controller.update_light(&mut self.light_model);
        self.light_uniform
            .update_view_proj(&self.light_model, &self.light_projection);
        self.light_uniform.strength = self.light_controller.strength;
        self.queue.write_buffer(
            &self.light_render_pass.uniform_buf,
            0,
            bytemuck::cast_slice(&[self.light_uniform]),
        );
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        // Create staging belt
        let mut staging_belt = wgpu::util::StagingBelt::new(1024);
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });
        encoder.push_debug_group("shadow passes");
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Shadow render pass"),
                color_attachments: &[],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.shadow_texture.view,
                    // view: &self.light_model.target_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            });
            pass.set_pipeline(&self.shadow_render_pass.pipeline);
            for chunk in self.chunks.iter() {
                pass.set_vertex_buffer(1, chunk.instance_buffer.slice(..));
                for mesh in &chunk.meshes {
                    pass.draw_light_mesh(
                        &mesh,
                        &self.camera_bind_group,
                        &self.shadow_render_pass.bind_group,
                    );
                }
            }
        }
        encoder.pop_debug_group();
        {
            let u32_size = std::mem::size_of::<u32>() as u32;
            let texture_size = Texture::SHADOW_SIZE.width;
            encoder.copy_texture_to_buffer(
                wgpu::ImageCopyTexture {
                    aspect: wgpu::TextureAspect::All,
                    texture: &self.shadow_texture.texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                },
                wgpu::ImageCopyBuffer {
                    buffer: &self.output_buffer,
                    layout: wgpu::ImageDataLayout {
                        offset: 0,
                        bytes_per_row: std::num::NonZeroU32::new(u32_size * texture_size),
                        rows_per_image: std::num::NonZeroU32::new(texture_size),
                    },
                },
                Texture::SHADOW_SIZE,
            );
        }
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(self.light_controller.color),
                        store: true,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            });
            // /
            // / RENDER LIGHT
            // /
            render_pass.set_pipeline(&self.light_render_pass.pipeline);
            for light_mesh in &self.light_model.meshes {
                render_pass.draw_light_mesh(
                    light_mesh,
                    &self.camera_bind_group,
                    &self.light_render_pass.bind_group,
                );
            }

            // /
            // / RENDER CHUNK
            // /
            render_pass.set_pipeline(&self.render_pipeline);
            for chunk in self.chunks.iter() {
                render_pass.set_vertex_buffer(1, chunk.instance_buffer.slice(..));
                for mesh in &chunk.meshes {
                    let material = &self.materials[mesh.material];
                    render_pass.draw_mesh(
                        &mesh,
                        &material,
                        &self.camera_bind_group,
                        &self.light_render_pass.bind_group,
                    );
                }
            }
        }
        // Queue text on top, it will be drawn first.
        // Depth buffer will make it appear on top.
        let sum: f32 = self.fps.iter().sum();
        let fps: f32 = self.fps.len() as f32 / if sum > 0.0 { sum } else { 1.0 };

        let look_at_coord = self.camera.look_at_coord();
        self.glyph_brush.queue(Section {
            screen_position: (30.0, 30.0),
            text: vec![Text::default()
                .with_text(
                    format!(
                        "FPS: {}\n\nCamera pos {:?}\n\nCamera target {:?}",
                        fps, self.camera.position, look_at_coord
                    )
                    .as_str(),
                )
                .with_scale(18.0)
                .with_color([0.8, 0.8, 0.8, 1.0])
                .with_z(0.9)],
            ..Section::default()
        });
        // Draw all the text!
        self.glyph_brush
            .draw_queued(
                &self.device,
                &mut staging_belt,
                &mut encoder,
                &view,
                wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(-1.0),
                        store: true,
                    }),
                    stencil_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(0),
                        store: true,
                    }),
                },
                self.size.width,
                self.size.height,
            )
            .expect("Draw queued");
        // Submit the work!
        staging_belt.finish();
        self.queue.submit(iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
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
