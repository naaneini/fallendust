use camera_controller::CameraController;
use ferrousgl::{GlWindow, WindowConfig};
use glam::Vec4;
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
            ..Default::default()
        }
    );

    let camera_controller = CameraController::new();
    let mut terrain_manager = TerrainManager::new();

    while !window.should_window_close() {
        window.clear_color(Vec4::new(0.0, 0.0, 0.0, 1.0));
        window.clear_depth();
        let title = format!("Fallendust (x64_win_shipping, {}/{})", window.get_frame_time(), 1000.0/144.0);
        window.set_window_title(&title);

        terrain_manager.render_terrain();

        window.update();
    }
}