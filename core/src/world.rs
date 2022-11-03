use cgmath::Point3;
use noise::Seedable;
use num_traits::{FromPrimitive, ToPrimitive};
use rand::RngCore;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};
use std::path::Path;

use crate::block::BlockKind;
use crate::chunk::Chunk;
use crate::texture;
use crate::utils;
#[derive(Debug, Serialize, Deserialize)]
pub struct World {
    chunks: Vec<BlockKind>,
}
impl World {
    /// A world size of SIZE x SIZE chunks
    pub const SIZE: usize = 10;
    /// A chunk has square base of CHUNK_WIDTH*CHUNK_WIDTH
    pub const CHUNK_WIDTH: usize = 16;
    /// Height of a chunk
    pub const CHUNK_HEIGHT: usize = 64;
    /// World voxel length
    pub const WORLD_SIZE: usize =
        Self::SIZE * Self::SIZE * Self::CHUNK_WIDTH * Self::CHUNK_WIDTH * Self::CHUNK_HEIGHT;

    pub fn generate(seed: u64) -> Self {
        let mut rng = ChaCha8Rng::seed_from_u64(seed);
        let perlin_noise = noise::Perlin::new().set_seed(seed as u32);
        let mut chunks = Vec::new();
        for _word_x in 0..Self::SIZE {
            for _word_z in 0..Self::SIZE {
                let mut chunk =
                    Self::generate_chunk(&mut rng, Self::CHUNK_WIDTH, Self::CHUNK_HEIGHT);
                chunks.append(&mut chunk);
            }
        }
        World { chunks }
    }
    pub fn generate_chunk<T>(rng: &mut T, width: usize, height: usize) -> Vec<BlockKind>
    where
        T: SeedableRng + RngCore,
    {
        let mut blocks = vec![];
        for y in 0..height {
            let max_y = rng.gen_range(height / 2..=height);
            for _x in 0..width {
                for _z in 0..width {
                    let block_type = if y > max_y {
                        0
                    } else if y < 2 {
                        1
                    } else {
                        let end = BlockKind::COUNT.to_i32().unwrap();
                        rng.gen_range(1..end)
                    };
                    blocks.push(BlockKind::from_i32(block_type).map_or(BlockKind::Air, |x| x));
                }
            }
        }
        blocks
    }
    pub fn load<P>(path: P) -> Self
    where
        P: AsRef<Path>,
    {
        todo!()
    }

    pub fn chunks(&self) -> &Vec<BlockKind> {
        &self.chunks
    }
    pub fn get_visible_chuncks(&self, Point3 { x, y, z }: Point3<i32>) -> &[BlockKind] {
        &self.chunks[0..1]
    }
}
