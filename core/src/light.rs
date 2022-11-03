use cgmath::{InnerSpace, Matrix4, Point3, Rad, Rotation3, Vector3};
use wgpu::util::DeviceExt;
use winit::event::{ElementState, VirtualKeyCode};

use crate::{
    camera::OPENGL_TO_WGPU_MATRIX,
    model::{self, Mesh},
    voxel,
};

#[derive(Debug)]
pub struct Light {
    pub position: Point3<f32>,
    pub meshes: Vec<Mesh>,
    // pub target_view: wgpu::TextureView,
}
impl Light {
    pub fn new(
        position: Point3<f32>,
        // target_view: wgpu::TextureView,
        device: &wgpu::Device,
    ) -> Light {
        let vertecies = voxel::CUBE.to_vec();
        let indecies: Vec<u32> = voxel::CUBE_INDICES.to_vec();
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("{:?}-{:?} Vertex Buffer", "Light Mesh", position)),
            contents: bytemuck::cast_slice(&vertecies),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("{:?} Index Buffer", "Light Mesh")),
            contents: bytemuck::cast_slice(&indecies),
            usage: wgpu::BufferUsages::INDEX,
        });
        let mesh = model::Mesh {
            name: "A Light Mesh".to_string(),
            vertex_buffer,
            index_buffer,
            num_elements: indecies.len() as u32,
            material: 0,
        };
        Light {
            position,
            meshes: vec![mesh],
            // target_view,
        }
    }
    pub fn calc_matrix(&self) -> Matrix4<f32> {
        let look_at = self.look_at_coord();
        Matrix4::look_to_rh(self.position, look_at, Vector3::unit_z())
    }
    pub fn look_at_coord(&self) -> Vector3<f32> {
        let pos: [f32; 3] = self.position.into();
        -Vector3::from(pos).normalize()
    }
}

#[derive(Debug)]
pub struct Projection {
    aspect: f32,
    fovy: Rad<f32>,
    znear: f32,
    zfar: f32,
    pub w: f32,
}

impl Projection {
    pub fn new<F: Into<Rad<f32>>>(width: u32, height: u32, fovy: F, znear: f32, zfar: f32) -> Self {
        Self {
            aspect: width as f32 / height as f32,
            fovy: fovy.into(),
            znear,
            zfar,
            w: 20.0,
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.aspect = width as f32 / height as f32;
    }

    pub fn calc_matrix(&self) -> Matrix4<f32> {
        let w = self.w;
        OPENGL_TO_WGPU_MATRIX * cgmath::ortho(-w, w, -w, w, self.znear, self.zfar)
        // OPENGL_TO_WGPU_MATRIX * cgmath::perspective(self.fovy, self.aspect, self.znear, self.zfar)
    }
}

#[derive(Debug, Default)]
pub struct LightController {
    pub strength: f32,
    step: f32,
    min: f32,
    max: f32,
    pub color: wgpu::Color,
    rotate: i8,
    pub orto_w: f32,
    auto_rotate: bool,
}
impl LightController {
    pub fn new(strength: f32, step: f32) -> Self {
        LightController {
            strength,
            step,
            min: 0.001,
            max: 20000.0,
            color: wgpu::Color {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 1.0,
            },
            rotate: 0,
            auto_rotate: false,
            orto_w: 20.0,
        }
    }
    pub fn process_keyboard(&mut self, key: VirtualKeyCode, state: ElementState) -> bool {
        println!(
            "key = {:?}, state = {:?}, self.auto_rotate = {:?}",
            &key, &state, &self.auto_rotate
        );
        match key {
            VirtualKeyCode::NumpadAdd => {
                self.strength = (self.strength + self.step).min(self.max);
                println!("VirtualKeyCode::Plus strength = {:?}", &self.strength);
                true
            }
            VirtualKeyCode::NumpadSubtract => {
                self.strength = (self.strength - self.step).max(self.min);
                println!("VirtualKeyCode::Minus strength = {:?}", &self.strength);
                true
            }

            VirtualKeyCode::Numpad0 => {
                self.color = wgpu::Color {
                    r: 0.2,
                    g: 0.2,
                    b: 0.2,
                    a: 1.0,
                };
                true
            }
            VirtualKeyCode::Numpad1 => {
                self.color = wgpu::Color {
                    r: 0.0,
                    g: 0.0,
                    b: 0.0,
                    a: 1.0,
                };
                true
            }
            VirtualKeyCode::Numpad2 => {
                self.color = wgpu::Color {
                    r: 0.2,
                    g: 0.2,
                    b: 0.2,
                    a: 1.0,
                };
                true
            }
            VirtualKeyCode::Numpad3 => {
                self.color = wgpu::Color {
                    r: 0.5,
                    g: 0.5,
                    b: 0.5,
                    a: 1.0,
                };
                true
            }
            VirtualKeyCode::Numpad4 => {
                self.color = wgpu::Color {
                    r: 0.75,
                    g: 0.75,
                    b: 0.75,
                    a: 1.0,
                };
                true
            }
            VirtualKeyCode::Numpad5 => {
                self.color = wgpu::Color {
                    r: 1.0,
                    g: 1.0,
                    b: 1.0,
                    a: 1.0,
                };
                true
            }
            VirtualKeyCode::RBracket => {
                if state == ElementState::Pressed {
                    self.rotate = 1;
                    self.auto_rotate = true;
                } else {
                    self.auto_rotate = false;
                }
                true
            }
            VirtualKeyCode::LBracket => {
                if state == ElementState::Pressed {
                    self.rotate = -1;
                    self.auto_rotate = true;
                } else {
                    self.auto_rotate = false;
                }
                true
            }
            VirtualKeyCode::P => {
                if state == ElementState::Pressed {
                    self.auto_rotate = !self.auto_rotate;
                }
                true
            }
            VirtualKeyCode::Key0 => {
                self.orto_w += 0.5;
                true
            }
            VirtualKeyCode::Key9 => {
                self.orto_w -= 0.5;
                true
            }
            VirtualKeyCode::Key8 => {
                self.orto_w = 2.0;
                true
            }
            VirtualKeyCode::Key7 => {
                self.orto_w = 20.0;
                true
            }
            VirtualKeyCode::Key6 => {
                self.orto_w = 200.0;
                true
            }
            VirtualKeyCode::Key5 => {
                self.orto_w = 2000.0;
                true
            }
            _ => false,
        }
    }
    pub fn update_light(&self, light: &mut Light) {
        if self.auto_rotate {
            let old_position: [f32; 3] = light.position.into();
            let old_position: Vector3<f32> = old_position.into();
            let Vector3 { x, y, z } = cgmath::Quaternion::from_axis_angle(
                (0.0, 0.4, 1.0).into(),
                cgmath::Deg(-0.07 * (self.rotate as f32)),
            ) * old_position;
            light.position = Point3::new(x, y, z)
        }
    }
}
