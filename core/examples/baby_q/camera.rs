use core::camera;
use core::utils;

use wgpu::util::DeviceExt;
#[derive(Debug)]
pub struct Camera {
    pub model: camera::Camera,
    pub projection: camera::Projection,
    pub controller: camera::CameraController,
    pub uniform: core::CameraUniform,
    pub buffer: wgpu::Buffer,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,
}
impl Camera {
    pub fn new(engine: &utils::Engine) -> Camera {
        let camera_camera =
            camera::Camera::new((-1.0, 17.0, -1.0), cgmath::Deg(45.0), cgmath::Deg(-70.0));
        let camera_projection = camera::Projection::new(
            engine.config.width,
            engine.config.height,
            cgmath::Deg(35.0),
            0.1,
            1000.0,
        );
        let camera_controller = camera::CameraController::new(4.0, 0.4);
        let mut camera_uniform = core::CameraUniform::new();
        camera_uniform.update_view_proj(&camera_camera, &camera_projection);

        let camera_buffer = engine
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Camera Buffer"),
                contents: bytemuck::cast_slice(&[camera_uniform]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });
        let camera_bind_group_layout =
            engine
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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

        let camera_bind_group = engine.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
            label: Some("camera_bind_group"),
        });
        Camera {
            model: camera_camera,
            projection: camera_projection,
            controller: camera_controller,
            uniform: camera_uniform,
            buffer: camera_buffer,
            bind_group_layout: camera_bind_group_layout,
            bind_group: camera_bind_group,
        }
    }
}
