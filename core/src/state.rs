use crate::data::{camera, light, shadow, sky};

use crate::chunk::Chunk;
use crate::instance;
use crate::model;
use crate::model::DrawLight;
use crate::model::DrawModel;
use crate::model::Vertex;
use crate::texture;
use cgmath::Vector3;
use std::iter;
use wgpu_glyph::{GlyphBrushBuilder, Section, Text};
use winit::event::ElementState;
use winit::event::KeyboardInput;
use winit::event::MouseButton;
use winit::event::WindowEvent;
#[derive(Debug)]
pub struct State {
    pub engine: crate::utils::Engine,
    pub camera: camera::Camera,
    pub light: light::Light,
    pub chunks: Vec<Chunk>,
    depth_texture: texture::Texture,
    pub mouse_pressed: bool,
    render_pipeline: wgpu::RenderPipeline,
    materials: Vec<model::Material>,
    shadow: shadow::Shadow,
    light_render_pipeline: wgpu::RenderPipeline,
    skybox: sky::Sky,
    glyph_brush: wgpu_glyph::GlyphBrush<wgpu::DepthStencilState>,
    fps: Vec<f32>,
}
impl State {
    pub fn new(engine: crate::utils::Engine) -> Self {
        let camera = camera::Camera::new(&engine);
        let light = light::Light::new(&engine);
        let shadow = shadow::Shadow::new(&light, &engine);
        let depth_texture =
            texture::Texture::create_depth_texture(&engine.device, &engine.config, "Depth Texture");
        let texture_bind_group_layout =
            engine
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
        let materials = crate::utils::load_materials(&engine, &texture_bind_group_layout);
        let mut chunks = vec![];
        for x in 0..15 {
            for z in 0..15 {
                let position = Vector3::new(
                    (x * Chunk::WIDTH) as f32 - 8.0,
                    0.0,
                    (z * Chunk::WIDTH) as f32 - 8.0,
                );
                let chunk = Chunk::new(position, &engine.device);
                chunks.push(chunk);
            }
        }
        let render_pipeline = {
            let layout = engine
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Render Pipeline Layout"),
                    bind_group_layouts: &[
                        &texture_bind_group_layout,
                        &camera.bind_group_layout,
                        &light.bind_group_layout,
                        &shadow.bind_group_layout,
                    ],
                    push_constant_ranges: &[],
                });
            let shader = wgpu::ShaderModuleDescriptor {
                label: Some("Normal Shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("shaders/shader.wgsl").into()),
            };
            crate::utils::create_render_pipeline(
                &engine.device,
                &layout,
                engine.config.format,
                Some(texture::Texture::DEPTH_FORMAT),
                &[model::ModelVertex::desc(), instance::InstanceRaw::desc()],
                shader,
                Some("Color Render Pipeline"),
            )
        };
        let light_render_pipeline = {
            let layout = engine
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Light Pipeline Layout"),
                    bind_group_layouts: &[&camera.bind_group_layout, &light.bind_group_layout],
                    push_constant_ranges: &[],
                });
            let shader = wgpu::ShaderModuleDescriptor {
                label: Some("Light Shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("shaders/light.wgsl").into()),
            };
            crate::utils::create_render_pipeline(
                &engine.device,
                &layout,
                engine.config.format,
                Some(texture::Texture::DEPTH_FORMAT),
                &[model::ModelVertex::desc()],
                shader,
                Some("Light Render Pipeline"),
            )
        };
        // {
        //     println!("light.uniform = {:?}", &light.uniform);
        //     println!("camera.uniform = {:?}", &camera.uniform);
        // }
        let out_dir = std::env::var("OUT_DIR").unwrap();
        println!("{:?}", &out_dir);
        let p = std::path::Path::new(&out_dir);
        // let root_path = &p.join("res").join("1k");
        let file_path = &p
            .join("res")
            .join("textures")
            .join("compressed")
            .join("mc_skybox.dds");
        let skybox = sky::Sky::new(file_path, &engine, &camera.bind_group_layout);

        let glyph_brush = {
            let font = wgpu_glyph::ab_glyph::FontArc::try_from_slice(include_bytes!(
                "../res/Roboto_Mono/static/RobotoMono-Regular.ttf"
            ))
            .unwrap();
            GlyphBrushBuilder::using_font(font)
                .depth_stencil_state(wgpu::DepthStencilState {
                    format: wgpu::TextureFormat::Depth32Float,
                    depth_write_enabled: true,
                    depth_compare: wgpu::CompareFunction::Greater,
                    stencil: wgpu::StencilState::default(),
                    bias: wgpu::DepthBiasState::default(),
                })
                .build(&engine.device, wgpu::TextureFormat::Bgra8UnormSrgb)
        };
        Self {
            engine,
            camera,
            light,
            chunks,
            depth_texture,
            mouse_pressed: false,
            render_pipeline,
            light_render_pipeline,
            materials,
            shadow,
            skybox,
            glyph_brush,
            fps: Vec::new(),
        }
    }
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
                let r = self.camera.controller.process_keyboard(*key, *state);
                let r = self.light.controller.process_keyboard(*key, *state) && r;
                r
            }
            WindowEvent::MouseWheel { delta, .. } => {
                self.camera.controller.process_scroll(delta);
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
    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.engine.config.width = new_size.width;
            self.engine.config.height = new_size.height;
            self.camera
                .projection
                .resize(new_size.width, new_size.height);
            self.engine
                .surface
                .configure(&self.engine.device, &self.engine.config);
            self.depth_texture = texture::Texture::create_depth_texture(
                &self.engine.device,
                &self.engine.config,
                "Color Depth Texture",
            );
        }
    }
    pub fn update(&mut self, dt: std::time::Duration) {
        if self.fps.len() > 120 {
            self.fps = self.fps.drain(1..120).collect();
        }
        self.fps.push(dt.as_secs_f32());

        self.camera
            .controller
            .update_camera(&mut self.camera.model, dt);
        self.camera
            .uniform
            .update_view_proj(&self.camera.model, &self.camera.projection);
        self.engine.queue.write_buffer(
            &self.camera.buffer,
            0,
            bytemuck::cast_slice(&[self.camera.uniform]),
        );
        // Update the light
        self.light.controller.update_light(&mut self.light.model);
        self.light.projection.w = self.light.controller.orto_w;
        self.light
            .uniform
            .update_view_proj(&self.light.model, &self.light.projection);
        self.light.uniform.strength = self.light.controller.strength;
        self.engine.queue.write_buffer(
            &self.light.buffer,
            0,
            bytemuck::cast_slice(&[self.light.uniform]),
        );
    }
    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        // Create staging belt
        let mut staging_belt = wgpu::util::StagingBelt::new(1024);
        let output = self.engine.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder =
            self.engine
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Render Encoder"),
                });
        {
            let mut shadow_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Shadow render pass"),
                color_attachments: &[],
                // color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                //     view: &view,
                //     resolve_target: None,
                //     ops: wgpu::Operations {
                //         load: wgpu::LoadOp::Clear(self.light.controller.color),
                //         store: true,
                //     },
                // })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.shadow.texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            });
            shadow_pass.set_pipeline(&self.shadow.pipeline);
            for chunk in self.chunks.iter() {
                shadow_pass.set_vertex_buffer(1, chunk.instance_buffer.slice(..));
                for mesh in &chunk.meshes {
                    shadow_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
                    shadow_pass
                        .set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                    shadow_pass.set_bind_group(0, &self.light.bind_group, &[]);
                    shadow_pass.draw_indexed(0..mesh.num_elements, 0, 0..1);
                }
            }
        }
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Color Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(self.light.controller.color),
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
            render_pass.set_pipeline(&self.light_render_pipeline);
            for light_mesh in &self.light.model.meshes {
                render_pass.draw_light_mesh(
                    light_mesh,
                    &self.camera.bind_group,
                    &self.light.bind_group,
                );
            }

            // /
            // / RENDER CHUNK
            // /
            render_pass.set_pipeline(&self.render_pipeline);
            for chunk in self.chunks.iter() {
                render_pass.set_vertex_buffer(1, chunk.instance_buffer.slice(..));
                render_pass.set_bind_group(3, &self.shadow.bind_group, &[]);
                for mesh in &chunk.meshes {
                    let material = &self.materials[mesh.material];

                    render_pass.draw_mesh(
                        &mesh,
                        &material,
                        &self.camera.bind_group,
                        &self.light.bind_group,
                    );
                }
            }
            // /
            // / SKYBOX REDNER
            // /
            render_pass.set_pipeline(&self.skybox.pipeline);
            render_pass.set_bind_group(0, &self.camera.bind_group, &[]);
            render_pass.set_bind_group(1, &self.skybox.bind_group, &[]);
            render_pass.draw(0..3, 0..1);
        }

        {
            // Queue text on top, it will be drawn first.
            // Depth buffer will make it appear on top.
            let sum: f32 = self.fps.iter().sum();
            let fps: f32 = self.fps.len() as f32 / if sum > 0.0 { sum } else { 1.0 };

            let look_at_coord = self.camera.model.look_at_coord();
            self.glyph_brush.queue(Section {
                screen_position: (30.0, 30.0),
                text: vec![Text::default()
                    .with_text(
                        format!(
                            "FPS: {}\n\nCamera pos {:?}\n\nCamera target {:?}",
                            fps, self.camera.model.position, look_at_coord
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
                    &self.engine.device,
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
                    self.engine.config.width,
                    self.engine.config.height,
                )
                .expect("Draw queued");
        }

        staging_belt.finish();
        self.engine.queue.submit(iter::once(encoder.finish()));
        output.present();
        Ok(())
    }
}
