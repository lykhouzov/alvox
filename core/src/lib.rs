use cgmath::{SquareMatrix, Vector3};
use light::Light;
use pollster::FutureExt;
pub use std::mem;
use winit::{
    dpi::PhysicalSize,
    event::{DeviceEvent, ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
#[macro_use]
extern crate lazy_static;

#[allow(dead_code)]
pub mod camera;
pub mod chunk;
pub mod data;
pub mod instance;
pub mod light;
#[allow(dead_code)]
pub mod model;
pub mod render;
#[allow(dead_code)]
pub mod resources;
pub mod state;
#[allow(dead_code)]
pub mod texture;
pub mod utils;
#[allow(dead_code)]
pub mod vertex;
#[allow(dead_code)]
pub mod voxel;
pub mod world;
pub mod block;
pub type Position = Vector3<f32>;

pub async fn run() {
    // let event_loop = EventLoop::new();
    // let window = WindowBuilder::new()
    //     .with_inner_size(PhysicalSize::new(1024, 768))
    //     .build(&event_loop)
    //     .unwrap();

    // // State::new uses async code, so we're going to wait for it to finish
    // let mut state = State::new(&window).await;
    // let mut last_render_time = instant::Instant::now();
    // event_loop.run(move |event, _, control_flow| {
    //     *control_flow = ControlFlow::Poll;
    //     match event {
    //         Event::MainEventsCleared => window.request_redraw(),
    //         // NEW!
    //         Event::DeviceEvent {
    //             event: DeviceEvent::MouseMotion{ delta, },
    //             .. // We're not using device_id currently
    //         } => if state.mouse_pressed {
    //             state.camera_controller.process_mouse(delta.0, delta.1)
    //         }
    //         Event::WindowEvent {
    //             ref event,
    //             window_id,
    //         } if window_id == window.id() && !state.input(event) => {
    //             match event {
    //                 #[cfg(not(target_arch="wasm32"))]
    //                 WindowEvent::CloseRequested
    //                 | WindowEvent::KeyboardInput {
    //                     input:
    //                         KeyboardInput {
    //                             state: ElementState::Pressed,
    //                             virtual_keycode: Some(VirtualKeyCode::Escape),
    //                             ..
    //                         },
    //                     ..
    //                 } => *control_flow = ControlFlow::Exit,
    //                 WindowEvent::Resized(physical_size) => {
    //                     state.resize(*physical_size);
    //                 }
    //                 WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
    //                     state.resize(**new_inner_size);
    //                 }
    //                 _ => {}
    //             }
    //         }
    //         Event::RedrawRequested(window_id) if window_id == window.id() => {
    //             let now = instant::Instant::now();
    //             let dt = now - last_render_time;
    //             last_render_time = now;
    //             state.update(dt);
    //             match state.render() {
    //                 Ok(_) => {}
    //                 // Reconfigure the surface if it's lost or outdated
    //                 Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => state.resize(state.size),
    //                 // The system is out of memory, we should probably quit
    //                 Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
    //                 // We're ignoring timeouts
    //                 Err(wgpu::SurfaceError::Timeout) => log::warn!("Surface timeout"),
    //             }
    //         }
    //         _ => {}
    //     }
    // });
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_inner_size(PhysicalSize::new(1024, 768))
        .build(&event_loop)
        .unwrap();
    let engine = async { utils::get_engine(&window).await }.block_on();
    let mut state = state::State::new(engine);
    // /
    // / EVENT LOOP
    // /
    let mut last_render_time = instant::Instant::now();
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;
        match event {
            Event::MainEventsCleared => window.request_redraw(),
            // NEW!
            Event::DeviceEvent {
                event: DeviceEvent::MouseMotion{ delta, },
                .. // We're not using device_id currently
            } => if state.mouse_pressed {
                state.camera.controller.process_mouse(delta.0, delta.1)
            }
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() && !state.input(event) => {
                match event {
                    #[cfg(not(target_arch="wasm32"))]
                    WindowEvent::CloseRequested
                    | WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                state: ElementState::Pressed,
                                virtual_keycode: Some(VirtualKeyCode::Escape),
                                ..
                            },
                        ..
                    } => *control_flow = ControlFlow::Exit,
                    WindowEvent::Resized(physical_size) => {
                        state.resize(*physical_size);
                    }
                    WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                        state.resize(**new_inner_size);
                    }
                    _ => {}
                }
            }
            Event::RedrawRequested(window_id) if window_id == window.id() => {
                let now = instant::Instant::now();
                let dt = now - last_render_time;
                last_render_time = now;
                state.update(dt);
                match state.render() {
                    Ok(_) => {}
                    // Reconfigure the surface if it's lost or outdated
                    Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => state.resize(window.inner_size()),
                    // The system is out of memory, we should probably quit
                    Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                    // We're ignoring timeouts
                    Err(wgpu::SurfaceError::Timeout) => log::warn!("Surface timeout"),
                }
            }
            _ => {}
        }
    });
}
#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable, Debug)]
pub struct CameraUniform {
    view_position: [f32; 4],
    view_proj: [[f32; 4]; 4],
    view: [[f32; 4]; 4],
    proj_inv: [[f32; 4]; 4],
}

impl CameraUniform {
    pub fn new() -> Self {
        Self {
            view_position: [0.0; 4],
            view_proj: cgmath::Matrix4::identity().into(),
            view: cgmath::Matrix4::identity().into(),
            proj_inv: cgmath::Matrix4::identity().into(),
        }
    }

    // UPDATED!
    pub fn update_view_proj(&mut self, camera: &camera::Camera, projection: &camera::Projection) {
        self.view_position = camera.position.to_homogeneous().into();
        let view = camera.calc_matrix();
        let proj = projection.calc_matrix();

        self.view_proj = (proj * view).into();
        self.proj_inv = proj.invert().unwrap().into();
        self.view = view.into();
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct LightUniform {
    pub view_position: [f32; 4],
    pub view_proj: [[f32; 4]; 4],
    pub color: [f32; 3],
    pub strength: f32,
}
impl LightUniform {
    pub fn new(strength: f32, color: [f32; 3]) -> Self {
        Self {
            view_position: [0.0; 4],
            view_proj: cgmath::Matrix4::identity().into(),
            strength,
            color,
        }
    }
    pub fn update_view_proj(&mut self, light: &Light, projection: &light::Projection) {
        self.view_position = light.position.to_homogeneous().into();
        self.view_proj = (projection.calc_matrix() * light.calc_matrix()).into()
    }
}
