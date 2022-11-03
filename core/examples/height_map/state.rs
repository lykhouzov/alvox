use cgmath::{Rotation3, Vector2, Vector3, Zero};
use core::chunk::Chunk;
use core::data::{camera, light, shadow};
use core::instance::{Instance, InstanceRaw};
use core::model;
use core::model::DrawLight;
use core::model::DrawModel;
use core::model::Vertex;
use core::texture;
use core::utils::Engine;
use core::world::World;
use core::{instance, voxel};
use egui::FontDefinitions;
use egui_winit_platform::{Platform, PlatformDescriptor};
use image::{DynamicImage, GenericImageView, ImageOutputFormat, Rgb, RgbImage};
use noise::{NoiseFn, Seedable};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use std::io::BufWriter;
use std::iter;
use std::num::NonZeroU32;
use wgpu::util::DeviceExt;
use winit::event::ElementState;
use winit::event::KeyboardInput;
use winit::event::MouseButton;
use winit::event_loop::EventLoop;
// use winit::event::VirtualKeyCode;
use winit::event::WindowEvent;

pub struct State {
    pub engine: core::utils::Engine,
    pub camera: camera::Camera,
    pub light: light::Light,
    depth_texture: texture::Texture,
    pub mouse_pressed: bool,
    render_pipeline: wgpu::RenderPipeline,
    materials: Vec<model::Material>,
    shadow: shadow::Shadow,
    light_render_pipeline: wgpu::RenderPipeline,
    pub world: World,
    plane: Plane,
    pub gui: Gui,
    pub platform: Platform,
}
impl State {
    pub fn new(engine: core::utils::Engine, event_loop: &EventLoop<()>) -> Self {
        let gui = Gui::new(&engine, event_loop);
        let (width, height) = (engine.config.width, engine.config.height);
        let platform = Platform::new(PlatformDescriptor {
            physical_width: width as u32,
            physical_height: height as u32,
            scale_factor: engine.scale_factor,
            font_definitions: FontDefinitions::default(),
            style: Default::default(),
        });
        let light = light::Light::new(&engine);
        let mut camera = camera::Camera::new(&engine);
        let look_at = Vector3::new(
            light.model.position.x,
            light.model.position.y,
            light.model.position.z,
        );
        camera.model.set_look_at(look_at);
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
        let materials = core::utils::load_materials(&engine, &texture_bind_group_layout);
        let world = World::generate(13);
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
        let plane = create_plane(&engine, Vector3::zero(), &camera);
        Self {
            platform,
            gui,
            engine,
            camera,
            light,
            world,
            depth_texture,
            mouse_pressed: false,
            render_pipeline,
            light_render_pipeline,
            materials,
            shadow,
            plane,
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
        // self.light.controller.update_light(&mut self.light.model);
        // self.light.projection.w = self.light.controller.orto_w;
        // self.light
        //     .uniform
        //     .update_view_proj(&self.light.model, &self.light.projection);
        // self.light.uniform.strength = self.light.controller.strength;
        // self.engine.queue.write_buffer(
        //     &self.light.buffer,
        //     0,
        //     bytemuck::cast_slice(&[self.light.uniform]),
        // );
        let TerrainWindow {
            lacunarity,
            octaves,
            persistance,
            scale,
            updated,
        } = self.gui.widget;
        let config = NoiseMapConfig {
            lacunarity,
            octaves,
            persistance,
            scale,
            ..Default::default()
        };
        let img = generate_heighmap_image(&self.engine, config);
        let dimensions = &img.dimensions();
        let texture = &self.plane.texture.texture;
        let data_layout = wgpu::ImageDataLayout {
            offset: 0,
            bytes_per_row: NonZeroU32::new(4 * dimensions.0),
            rows_per_image: NonZeroU32::new(dimensions.1),
        };
        let size = wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: 1,
        };
        self.engine
            .queue
            .write_texture(texture.as_image_copy(), &img.to_rgba8(), data_layout, size)
    }
    pub fn render(&mut self, window: &winit::window::Window) -> Result<(), wgpu::SurfaceError> {
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
            // / RENDER HEIGHTMAP
            // /
            render_pass.set_pipeline(&self.plane.render_pipeline);
            render_pass.set_vertex_buffer(0, self.plane.vertex_buffer.slice(..));
            render_pass.set_vertex_buffer(1, self.plane.instance_buffer.slice(..));
            render_pass
                .set_index_buffer(self.plane.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            render_pass.set_bind_group(0, &self.camera.bind_group, &[]);
            render_pass.set_bind_group(1, &self.plane.bind_group, &[]);
            render_pass.draw_indexed(0..self.plane.num_elements, 0, 0..1);
        }

        {
            self.platform.begin_frame();
            self.gui.widget.show(&self.platform.context(), &mut true);
            let screen_descriptor = egui_wgpu::renderer::ScreenDescriptor {
                size_in_pixels: [self.engine.config.width, self.engine.config.height],
                pixels_per_point: window.scale_factor() as f32,
            };
            // / GUI
            let full_output = self.platform.end_frame(Some(&window));
            let paint_jobs = self.platform.context().tessellate(full_output.shapes);
            for (id, image_delta) in &full_output.textures_delta.set {
                self.gui.render_pass.update_texture(
                    &self.engine.device,
                    &self.engine.queue,
                    *id,
                    image_delta,
                );
            }
            for id in &full_output.textures_delta.free {
                self.gui.render_pass.free_texture(id);
            }
            self.gui.render_pass.update_buffers(
                &self.engine.device,
                &self.engine.queue,
                &paint_jobs,
                &screen_descriptor,
            );
            // self.gui.render_pass.execute_with_renderpass(rpass, paint_jobs, screen_descriptor)
            // Record all render passes.
            self.gui.render_pass.execute(
                &mut encoder,
                &view,
                &paint_jobs,
                &screen_descriptor,
                None,
            );
        }
        // {
        //     egui::CentralPanel::default().show(&self.gui.context, |ui| {
        //         egui::ScrollArea::both()
        //             .auto_shrink([false; 2])
        //             .show(ui, |ui| {
        //                 ui.horizontal(|ui| {
        //                     ui.spacing_mut().item_spacing.x = 0.0;
        //                     ui.label("The triangle is being painted using ");
        //                     ui.hyperlink_to("WGPU", "https://wgpu.rs");
        //                     ui.label(" (Portable Rust graphics API awesomeness)");
        //                 });
        //                 ui.label("It's not a very impressive demo, but it shows you can embed 3D inside of egui.");

        //                 egui::Frame::canvas(ui.style()).show(ui, |ui| {
        //                     // self.custom_painting(ui);
        //                 });
        //                 ui.label("Drag to rotate!");
        //                 // ui.add(egui_demo_lib::egui_github_link_file!());
        //             });
        //     });
        // }

        staging_belt.finish();
        self.engine.queue.submit(iter::once(encoder.finish()));
        output.present();
        Ok(())
    }
}
#[derive(Debug)]
pub struct Plane {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub instance_buffer: wgpu::Buffer,
    render_pipeline: wgpu::RenderPipeline,
    num_elements: u32,
    texture: texture::Texture,
    bind_group: wgpu::BindGroup,
}
pub fn create_plane(engine: &Engine, position: Vector3<f32>, camera: &camera::Camera) -> Plane {
    let vertices = voxel::PLANE.to_vec(); //voxel::Voxel::FACE_TOP;
    let indices = voxel::PLANE_INDICES.to_vec(); //vec![0, 3, 1, 1, 3, 2];
                                                 // let vertices = voxel::CUBE.to_vec();
                                                 // let indices = voxel::CUBE_INDICES.to_vec();
    let vertex_buffer = engine
        .device
        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Instance Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });
    let index_buffer = engine
        .device
        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("{:?} Index Buffer", "Chunk 0")),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        });
    let rotation = cgmath::Quaternion::from_axis_angle(cgmath::Vector3::unit_z(), cgmath::Deg(0.0));
    let scale = 14.0; //(World::CHUNK_WIDTH * World::WORLD_SIZE) as f32;
    let model = cgmath::Matrix4::from_translation(position)
        * cgmath::Matrix4::from(rotation)
        * cgmath::Matrix4::from_scale(scale);
    let instance_raw = InstanceRaw {
        model: model.into(),
        normal: cgmath::Matrix3::from(rotation).into(),
    };
    let instance_buffer = engine
        .device
        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Instance Buffer"),
            contents: bytemuck::cast_slice(&[instance_raw]),
            usage: wgpu::BufferUsages::VERTEX,
        });
    let num_elements = indices.len() as u32;
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
                ],
                label: Some("texture_bind_group_layout"),
            });

    let texture = create_noise_texture(engine, Default::default());
    let bind_group = engine.device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &texture_bind_group_layout,
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
        label: None,
    });
    let render_pipeline = {
        let layout = engine
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Light Pipeline Layout"),
                bind_group_layouts: &[&camera.bind_group_layout, &texture_bind_group_layout],
                push_constant_ranges: &[],
            });
        let shader = wgpu::ShaderModuleDescriptor {
            label: Some("Heightmap Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/heightmap.wgsl").into()),
        };
        crate::utils::create_render_pipeline(
            &engine.device,
            &layout,
            engine.config.format,
            Some(texture::Texture::DEPTH_FORMAT),
            &[model::ModelVertex::desc(), InstanceRaw::desc()],
            shader,
            Some("Plane Render Pipeline"),
        )
    };
    Plane {
        vertex_buffer,
        index_buffer,
        instance_buffer,
        num_elements,
        render_pipeline,
        texture,
        bind_group,
    }
}
pub fn inverse_lerp(a: f64, b: f64, v: f64) -> f64 {
    (v - a) / (b - a)
}
pub fn generate_heighmap_image(engine: &Engine, config: NoiseMapConfig) -> DynamicImage {
    let mut rng = ChaCha8Rng::seed_from_u64(config.seed);
    let perlin_noise = noise::Perlin::new().set_seed(config.seed as u32);
    let (width, height, scale) = (config.width, config.height, config.scale);

    let mut octave_offsets: Vec<Vector2<f64>> = vec![];
    let offset: Vector2<f64> = Vector2::new(0.0, 0.0);
    for _ in 0..config.octaves {
        let offset_x = rng.gen_range(-100000.0..100000.0) + offset.x;
        let offset_y = rng.gen_range(-100000.0..100000.0) + offset.y;
        octave_offsets.push(Vector2::new(offset_x, offset_y));
    }
    let mut max_noise_height = std::f64::MIN;
    let mut min_noise_height = std::f64::MAX;
    let (half_width, half_height) = config.get_half_dimentions();
    let mut noise_map = vec![0.0; (width * height) as usize];

    for x in 0..width {
        for y in 0..height {
            let mut amplitude = 1.0;
            let mut frequency = 1.0;
            let mut noise_height = 0.0;
            for i in 0..config.octaves {
                let sample_x = (x as f64 - half_width) / scale * frequency + octave_offsets[i].x;
                let sample_y = (y as f64 - half_height) / scale * frequency + octave_offsets[i].y;
                let noise_value = perlin_noise.get([sample_x, sample_y]);

                noise_height += noise_value * amplitude;

                amplitude *= config.persistance;
                frequency *= config.lacunarity;
            }

            if noise_height > max_noise_height {
                max_noise_height = noise_height;
            } else if noise_height < min_noise_height {
                min_noise_height = noise_height;
            }
            noise_map[x as usize + (y * width) as usize] = noise_height;
        }
    }
    let mut imgbuf = image::GrayImage::new(width, height);
    for (x, y, pixel) in imgbuf.enumerate_pixels_mut() {
        let i = x as usize + y as usize * (width as usize);
        let val: f64 = inverse_lerp(min_noise_height, max_noise_height, noise_map[i]);
        // let val = noise_map[i];
        let pixel_color = (val * 255.0) as u8;

        *pixel = image::Luma([pixel_color]);
    }

    DynamicImage::ImageLuma8(imgbuf)
}
pub fn create_noise_texture(engine: &Engine, config: NoiseMapConfig) -> texture::Texture {
    let img = generate_heighmap_image(engine, config);
    texture::Texture::from_image(
        &engine.device,
        &engine.queue,
        &img,
        Some("noise image texture"),
        false,
    )
    .unwrap()
}

pub struct NoiseMapConfig {
    pub width: u32,
    pub height: u32,
    pub seed: u64,
    pub scale: f64,
    pub octaves: usize,
    pub persistance: f64,
    pub lacunarity: f64,
}
impl Default for NoiseMapConfig {
    fn default() -> Self {
        NoiseMapConfig {
            width: (World::CHUNK_WIDTH * World::SIZE) as u32,
            height: (World::CHUNK_WIDTH * World::SIZE) as u32,
            seed: 13,
            scale: 27.0,
            octaves: 4,
            persistance: 0.5,
            lacunarity: 2.0,
        }
    }
}
impl NoiseMapConfig {
    pub fn get_half_dimentions(&self) -> (f64, f64) {
        (self.width as f64 / 2.0, self.height as f64 / 2.0)
    }
}

pub struct Gui {
    pub state: egui_winit::State,
    pub widget: TerrainWindow,
    pub render_pass: egui_wgpu::renderer::RenderPass,
}

impl std::fmt::Debug for Gui {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Gui")
            .field("state", &"self.state")
            .field("context", &"self.context")
            .finish()
    }
}
impl Gui {
    pub fn new(engine: &Engine, event_loop: &EventLoop<()>) -> Self {
        // We use the egui_winit_platform crate as the platform.
        let state = egui_winit::State::new(event_loop);

        let render_pass =
            egui_wgpu::renderer::RenderPass::new(&engine.device, engine.config.format, 1);
        let widget = TerrainWindow::new();
        Self {
            state,
            widget,
            render_pass,
        }
    }
}
pub struct TerrainWindow {
    pub scale: f64,
    pub octaves: usize,
    pub persistance: f64,
    pub lacunarity: f64,
    pub updated: bool,
}
impl TerrainWindow {
    pub fn new() -> Self {
        let NoiseMapConfig {
            scale,
            octaves,
            persistance,
            lacunarity,
            height,
            width,
            seed,
        } = NoiseMapConfig::default();
        Self {
            scale,
            octaves,
            persistance,
            lacunarity,
            updated: false,
        }
    }
    pub fn show(&mut self, ctx: &egui::Context, open: &mut bool) {
        egui::Window::new("Noise Generator")
            .open(open)
            .resizable(false)
            .show(ctx, |ui| {
                self.ui(ui);
            });
    }
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        let Self {
            scale,
            octaves,
            persistance,
            lacunarity,
            updated,
        } = self;

        ui.label("You can click a slider value to edit it with the keyboard.");

        ui.add(
            egui::Slider::new(scale, 1.0..=500.0)
                .orientation(egui::SliderOrientation::Horizontal)
                .text("Scale")
                .step_by(0.1),
        );
        ui.add(
            egui::Slider::new(octaves, 1..=6)
                .orientation(egui::SliderOrientation::Horizontal)
                .integer()
                .text("Octaves")
                .step_by(1.0),
        );
        ui.add(
            egui::Slider::new(persistance, -50.0..=50.0)
                .orientation(egui::SliderOrientation::Horizontal)
                .text("Persistance")
                .step_by(0.1),
        );
        ui.add(
            egui::Slider::new(lacunarity, -50.0..=50.0)
                .orientation(egui::SliderOrientation::Horizontal)
                .text("Lacunarity")
                .step_by(0.1),
        );

        if ui.button("Reset").clicked() {
            let NoiseMapConfig {
                scale,
                octaves,
                persistance,
                lacunarity,
                height,
                width,
                seed,
            } = NoiseMapConfig::default();
            self.scale = scale;
            self.octaves = octaves;
            self.persistance = persistance;
            self.lacunarity = lacunarity;
        }

        ui.separator();
    }
}
