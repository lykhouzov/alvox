use crate::model;

pub trait DrawChunk<'a> {
    fn draw_chunk_mesh(
        &mut self,
        mesh: &'a model::Mesh,
        camera_bind_group: &'a wgpu::BindGroup,
        light_bind_group: &'a wgpu::BindGroup,
    );
}