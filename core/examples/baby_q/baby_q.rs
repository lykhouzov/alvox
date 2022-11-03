pub struct Scene {}
impl Scene {
    pub fn new() -> Self {
        Self {}
    }
    pub fn add_camera(&mut self, camera: camera::Camera) {
        todo!()
    }

    pub(crate) fn add_light(&mut self, light: light::Light) {
        todo!()
    }
}
pub mod camera {
    pub struct Camera {}
    impl Camera {
        pub fn new() -> Self {
            Self {}
        }
    }
}
pub mod light {
    pub struct Light {}
    impl Light {
        pub fn new() -> Self {
            Self {}
        }
    }
}
pub mod mesh {}

pub fn run() {
    let mut scene = Scene::new();
    let camera = camera::Camera::new();
    scene.add_camera(camera);
    let light = light::Light::new();
    scene.add_light(light);
}
