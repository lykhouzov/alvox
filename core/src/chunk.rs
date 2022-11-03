pub mod traits;

use std::usize;

use cgmath::Rotation3;
use noise::{NoiseFn, Seedable};
use wgpu::util::DeviceExt;

use crate::model::ModelVertex;
use crate::{
    instance::Instance,
    model::{self, Mesh},
    texture, utils,
    voxel::Voxel,
    Position,
};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
#[derive(Debug)]
pub struct Chunk {
    pub position: Position,
    pub meshes: Vec<Mesh>,
    pub instance_buffer: wgpu::Buffer,
    pub instance_num: u32,
    // pub instance_buffers: Vec<(u32, Mesh, wgpu::Buffer)>,
}
lazy_static! {
    #[derive(Debug)]
    static ref VOXELS: [[[u8; Chunk::WIDTH]; Chunk::HEIGHT]; Chunk::WIDTH] = {
        let mut rnd = rand::thread_rng();
        let mut map = [[[0; Chunk::WIDTH]; Chunk::HEIGHT]; Chunk::WIDTH];
        for y in 0..Chunk::HEIGHT {
            for x in 0..Chunk::WIDTH {
                for z in 0..Chunk::WIDTH {
                    map[x][y][z] = rnd.gen_range(0..3);
                }
            }
        }
        map
    };
}
type MapMatrix = [usize; Chunk::WIDTH * Chunk::HEIGHT * Chunk::WIDTH];

impl Chunk {
    pub const WIDTH: usize = 16;
    pub const HEIGHT: usize = 64;
    const NUM_BLOCK_TYPES: usize = texture::TEXTURE_NAMES.len();
    const VOXEL_MAP: &'static MapMatrix = &[1; Chunk::WIDTH * Chunk::HEIGHT * Chunk::WIDTH];
    pub fn generate(seed: u64, offset: &cgmath::Vector3<f32>) -> MapMatrix {
        let mut rng = ChaCha8Rng::seed_from_u64(seed);
        let mut map = [0; Chunk::WIDTH * Chunk::HEIGHT * Chunk::WIDTH];
        let perlin_noise = noise::Perlin::new().set_seed(seed as u32);
        let mut max_value = 0.0;
        for y in 0..Chunk::HEIGHT {
            for x in 0..Chunk::WIDTH {
                for z in 0..Chunk::WIDTH {
                    let p_x = (x as f64 + offset.x as f64) / (Chunk::WIDTH as f64);
                    let p_z = (z as f64 + offset.z as f64) / (Chunk::WIDTH as f64);
                    let p_y = (y as f64 + offset.y as f64) / (Chunk::HEIGHT as f64);
                    let noise_value = (perlin_noise.get([p_x, p_y, p_z]) + 1.0) / 2.0;
                    let max_y: f64 = noise_value * (Chunk::HEIGHT as f64);
                    let max_y = max_y.floor() as usize;
                    let position = Position::new(x as f32, y as f32, z as f32);
                    // max_value = max_y.max(max_value);
                    // println!("max_y = {:#?} at {:?}", &max_y, &position);
                    let i = utils::to_index(&position);
                    if y == 0 {
                        map[i] = 1;
                    } else if y > max_y {
                        map[i] = 0;
                    } else {
                        // println!("y = {}", &y);
                        map[i] = match y {
                            0..=20 => 4,
                            21..=25 => {
                                if rng.gen_range(0..=1) == 0 {
                                    4
                                } else {
                                    6
                                }
                            }
                            26..=54 => 6,
                            55..=63 => 5,
                            _ => rng.gen_range(1..5),
                        }
                    }
                }
            }
        }
        // println!("max_value = {:?}", &max_value);
        map
    }
    pub fn new(position: Position, device: &wgpu::Device) -> Self {
        log::trace!("a Chunk position = {:?}", position);
        let seed = 1982;
        let map = Chunk::generate(seed, &position);
        log::trace!("map is generated");
        let mut indices = vec![Vec::<u32>::new(); Chunk::NUM_BLOCK_TYPES];
        let mut vertices = vec![Vec::<ModelVertex>::new(); Chunk::NUM_BLOCK_TYPES];
        log::trace!("prepare vertices");
        for y in 0..Chunk::HEIGHT {
            for x in 0..Chunk::WIDTH {
                for z in 0..Chunk::WIDTH {
                    /* @todo use a block type
                    for the moment map[x][y][z] contains material ID,
                    but ideally it should be a block type,
                    which contains information about material */
                    let vox_position = Position::new(x as f32, y as f32, z as f32);
                    let i = utils::to_index(&vox_position);
                    let material_id = map[i];
                    // @todo use a block type and do not render for invisible blocks
                    if material_id == 0 {
                        continue;
                    }
                    let material_id = material_id - 1; // @todo that should be a block type
                                                       // Let's generate a voxel vertices and indices avoiding invisible faces
                    let (vox_indices, mut vox_vertices) =
                        Chunk::add_voxel_to_chunk(&vox_position, &map);
                    log::trace!("voxel is generated {:?}", &vox_position);
                    let vox_idx = vertices[material_id].len();
                    vertices[material_id].append(&mut vox_vertices);
                    let mut vox_indices = vox_indices
                        .iter()
                        .map(|i| (i + vox_idx) as u32)
                        .collect::<Vec<_>>();
                    indices[material_id].append(&mut vox_indices);
                }
            }
        }
        log::trace!("voxels are added");
        // let vox_pos = Vector3::zero();
        // let (vox_indices, vox_vertices) = Chunk::add_voxel_to_chunk(vox_pos, &map);
        // vertices[0] = vox_vertices;
        // let vox_indices: Vec<u32> = vox_indices.iter().map(|i| *i as u32).collect();
        // indices[0] = vox_indices;

        // generate meshes
        let mut meshes = Vec::new();
        for block_type_id in 0..Chunk::NUM_BLOCK_TYPES {
            let (mesh_vertices, mesh_indices) = (&vertices[block_type_id], &indices[block_type_id]);
            if mesh_indices.len() == 0 || mesh_vertices.len() == 0 {
                continue;
            }
            let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Instance Buffer"),
                contents: bytemuck::cast_slice(mesh_vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });
            let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&format!("{:?} Index Buffer", "Chunk 0")),
                contents: bytemuck::cast_slice(&mesh_indices),
                usage: wgpu::BufferUsages::INDEX,
            });
            meshes.push(model::Mesh {
                name: "A mesh".to_string(),
                vertex_buffer,
                index_buffer,
                num_elements: mesh_indices.len() as u32,
                material: block_type_id,
            });
        }

        let rotation =
            cgmath::Quaternion::from_axis_angle(cgmath::Vector3::unit_z(), cgmath::Deg(0.0));
        let instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Instance Buffer"),
            contents: bytemuck::cast_slice(&[(Instance { position, rotation }).to_raw()]),
            usage: wgpu::BufferUsages::VERTEX,
        });
        Self {
            position,
            meshes,
            instance_buffer,
            instance_num: 1,
            // instance_buffers,
        }
    }
    /// Returns vertex indices which ara alowed to be drawn
    pub fn add_voxel_to_chunk(
        position: &Position,
        map: &MapMatrix,
    ) -> (Vec<usize>, Vec<model::ModelVertex>) {
        let mut indices = Vec::<usize>::new();
        let mut vertices = Vec::<model::ModelVertex>::new();
        for (face_id, face) in Voxel::FACES.iter().enumerate() {
            log::trace!("check face {}", face_id);
            // calculate next to the ace position of a voxel
            let check_pos = &Voxel::FACE_CHECK[face_id] + position;
            log::trace!("check position {:?}", &check_pos);
            let i = utils::to_index(&check_pos);
            if Chunk::check_voxel(
                &check_pos,
                map.len() > i && (map[i] > 0 && map[i] < Chunk::NUM_BLOCK_TYPES),
            ) {
                log::trace!("face #{:?} is not allowed to be drawn", &face_id);
                continue;
            }
            log::trace!("check face {} passed", face_id);
            let idx = vertices.len();
            log::trace!("current vertices len {}", idx);
            vertices.push(face[0].translate(&position));
            log::trace!("v0 pushed");
            vertices.push(face[1].translate(&position));
            log::trace!("v1 pushed");
            vertices.push(face[2].translate(&position));
            log::trace!("v2 pushed");
            vertices.push(face[3].translate(&position));
            log::trace!("v3 pushed");

            indices.push(0 + idx);
            log::trace!("i0 pushed");
            indices.push(1 + idx);
            log::trace!("i1 pushed");
            indices.push(2 + idx);
            log::trace!("i2 pushed");
            indices.push(2 + idx);
            log::trace!("i3 pushed");
            indices.push(3 + idx);
            log::trace!("i4 pushed");
            indices.push(0 + idx);
            log::trace!("i5 pushed");
        }
        // Re calculate normals, tangents and bitangents
        // indices.chunks(3).for_each(|ids| {
        //     let v0 = &vertices[ids[0]];
        //     let v1 = &vertices[ids[1]];
        //     let v2 = &vertices[ids[2]];
        //     let (normals, tangents, bitangents) = utils::calc_normals(&[v0, v1, v2]);
        //     for (((i, normal), tangent), bitangent) in ids
        //         .iter()
        //         .zip(normals.iter())
        //         .zip(tangents.iter())
        //         .zip(bitangents.iter())
        //     {
        //         if let Some(v) = vertices.get_mut(*i) {
        //             v.normal = *normal;
        //             v.tangent = *tangent;
        //             v.bitangent = *bitangent;
        //         }
        //     }
        // });
        (indices, vertices)
    }
    pub fn check_voxel(pos: &Position, map: bool) -> bool {
        log::trace!("Start checking position of a neighbor block");
        let x = pos.x;
        let y = pos.y;
        let z = pos.z;
        if x < 0.0
            || x > (Self::WIDTH - 1) as f32
            || y < 0.0
            || y > (Self::HEIGHT - 1) as f32
            || z < 0.0
            || z > (Self::WIDTH - 1) as f32
        {
            return false;
        }
        let i = utils::to_index(pos);
        return Self::VOXEL_MAP[i] == 1 && map;
        // return VOXELS[x][y][z] > 0;
    }
}
