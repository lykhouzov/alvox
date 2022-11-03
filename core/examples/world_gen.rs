use core::{block::BlockKind, world::World};
use std::fs;
pub fn main() {
    let start = instant::Instant::now();
    let world = World::generate(13);

    let duration = instant::Instant::now() - start;
    println!(
        "World of size {:?} voxels was generated during {} seconds",
        World::WORLD_SIZE,
        duration.as_secs_f32()
    );
    println!(
        "Size of world is {:#?}",
        std::mem::size_of::<BlockKind>() * world.chunks().len()
    );
    println!("START SAVE TO FILE");
    let start = instant::Instant::now();
    let file = fs::OpenOptions::new()
        .write(true)
        .create(true)
        .open("world.json")
        .unwrap();
    serde_json::to_writer(file, &world).unwrap();
    let duration = instant::Instant::now() - start;
    println!("DONE: {:?}", duration.as_secs_f32());
}
