use core::{light, utils};

use wgpu::util::DeviceExt;
#[derive(Debug)]
pub struct Light {
    pub model: light::Light,
    pub projection: light::Projection,
    pub controller: light::LightController,
    pub uniform: core::LightUniform,
    pub buffer: wgpu::Buffer,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,
}
impl Light {
    pub fn new(engine: &utils::Engine) -> Light {
        let light_model = light::Light::new((-60.0, 8.0, -60.0).into(), &engine.device);
        let light_projection = light::Projection::new(
            engine.config.width,
            engine.config.height,
            cgmath::Deg(60.0),
            20.0,
            2000.0,
        );
        let mut light_uniform = core::LightUniform::new(1000.0, [1.0, 1.0, 1.0]);
        light_uniform.update_view_proj(&light_model, &light_projection);
        let light_buffer = engine
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Light VB"),
                contents: bytemuck::cast_slice(&[light_uniform]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });
        let light_bind_group_layout =
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
                    label: None,
                });
        let light_bind_group = engine.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &light_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: light_buffer.as_entire_binding(),
            }],
            label: None,
        });
        let light_controller = light::LightController::new(180.0, 10.0);
        Light {
            model: light_model,
            projection: light_projection,
            controller: light_controller,
            uniform: light_uniform,
            buffer: light_buffer,
            bind_group_layout: light_bind_group_layout,
            bind_group: light_bind_group,
        }
    }
}
