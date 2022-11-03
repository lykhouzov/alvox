const NUM_INSTANCES_PER_ROW: usize = 2;
const SPACE_BETWEEN: f32 = 1.0;
fn main(){

    let instances:Vec<_> = (0..NUM_INSTANCES_PER_ROW)
    .flat_map(|z| {
        (0..NUM_INSTANCES_PER_ROW).map(move |x| {
            let x = SPACE_BETWEEN * (x as f32 - NUM_INSTANCES_PER_ROW as f32 );
            let z = SPACE_BETWEEN * (z as f32 - NUM_INSTANCES_PER_ROW as f32 );

            let position = cgmath::Vector3 { x, y: 0.0, z };
            position
        })
    }).collect();
    println!("{:?}", instances);

}