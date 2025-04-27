use std::path::Path;

use camera_controller::CameraController;
use ferrousgl::{GlWindow, Mesh, Shader, WindowConfig};
use glam::{Mat4, Vec3, Vec4, IVec3};
use terrain::terrain_manager::TerrainManager;

mod terrain;
mod camera_controller;

fn main() {
    let mut window = GlWindow::new(
        WindowConfig {
            title: "Fallendust".to_string(),
            width: 1080,
            height: 720,
            target_framerate: 144,
            anti_aliasing: 4,
            ..Default::default()
        }
    );

    let mut camera_controller = CameraController::new((window.get_window_size().0 as f32) / (window.get_window_size().1 as f32));
    let mut terrain_manager = TerrainManager::new();

    let seed = 123456789;
    let frequency = 0.5;    

    terrain_manager.enqueue_chunks_in_radius(IVec3::new(0, 0, 0), 4);

    while !window.should_window_close() {
        window.clear_color(Vec4::new(0.4, 0.4, 0.9, 1.0));
        window.clear_depth();

        let vp = camera_controller.get_vp();
        camera_controller.update(&mut window);

        // Process one chunk per frame
        terrain_manager.process_chunk_generation(69420, 0.5);

        terrain_manager.render(&mut window, vp);

        let title = format!(
            "Fallendust x64_win_debug_build, FT Budget {}µs [{:.1}%], cam_pos:[{}] gl_ver[{}], gl_vendor[{}], accelerator device(discrete/integrated)[{}]",
            window.get_frame_time(),
            (window.get_frame_time() as f64 / ((1.0 / 144.0) * 1_000_000.0)) * 100.0,
            camera_controller.position,
            unsafe {
                window.get_opengl_ver() + &window.get_glsl_ver()
            },
            unsafe {
                window.get_vendor()
            },
            unsafe {
                window.get_renderer()
            }
        );
        window.set_window_title(&title);
        window.update();
    }
}