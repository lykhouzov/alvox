use num_derive::{FromPrimitive, ToPrimitive};
use serde::{Deserialize, Serialize};
use serde_repr::*;
#[derive(Debug, Default, FromPrimitive, ToPrimitive, Serialize_repr, Deserialize_repr)]
#[repr(u16)]
pub enum BlockKind {
    #[default]
    Air = 0,
    Stone,
    Granite,
    Diorite,
    Andesite,
    Grass,
    Dirt,
    //
    COUNT,
}
impl BlockKind {
    pub fn len() -> usize {
        Self::COUNT as usize
    }
}

impl BlockKind {}
#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Block {
    kind: BlockKind,
}
