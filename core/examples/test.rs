use core::utils;

use pollster::FutureExt;
use wgpu::{AstcBlock, AstcChannel};
use winit::{dpi::PhysicalSize, event_loop::EventLoop, window::WindowBuilder};
pub fn main() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_inner_size(PhysicalSize::new(1024, 768))
        .build(&event_loop)
        .unwrap();
    let engine = async { utils::get_engine(&window).await }.block_on();
    let device_features = engine.device.features();
    let skybox_format = if device_features.contains(wgpu::Features::TEXTURE_COMPRESSION_ASTC_LDR) {
        // wgpu::TextureFormat::Astc4x4RgbaUnormSrgb
        wgpu::TextureFormat::Astc {
            block: AstcBlock::B4x4,
            channel: AstcChannel::UnormSrgb,
        }
    } else if device_features.contains(wgpu::Features::TEXTURE_COMPRESSION_ETC2) {
        wgpu::TextureFormat::Etc2Rgb8UnormSrgb
    } else if device_features.contains(wgpu::Features::TEXTURE_COMPRESSION_BC) {
        wgpu::TextureFormat::Bc1RgbaUnormSrgb
    } else {
        wgpu::TextureFormat::Bgra8UnormSrgb
    };
    println!("{:?}", skybox_format);
}
