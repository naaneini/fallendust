use std::path::Path;

use camera_controller::CameraController;
use ferrousgl::{DepthType, GlWindow, Mesh, RenderTexture, Shader, WindowConfig, WindowKey};
use glam::{IVec3, Mat4, Vec3, Vec4};
use terrain::terrain_manager::TerrainManager;

mod camera_controller;
mod terrain;
mod utils;

fn main() {
    println!("Initializing Core Engine");

    let mut window = GlWindow::new(WindowConfig {
        title: "Fallendust".to_string(),
        width: 1080,
        height: 720,
        target_framerate: 144,
        anti_aliasing: 4,
        ..Default::default()
    });

    let mut camera_controller = CameraController::new(
        (window.get_window_size().0 as f32) / (window.get_window_size().1 as f32),
    );
    let mut terrain_manager = TerrainManager::new();

    let seed = 123456789;
    let frequency = 0.5;

    terrain_manager.enqueue_chunks_in_radius(IVec3::new(0, 0, 0), 8);
    
    // shadow stuff
    let depth_shader = Shader::new_from_file(
        Path::new("./assets/shaders/shadows/vertex.glsl"),
        Path::new("./assets/shaders/shadows/fragment.glsl"),
    ).unwrap();

    let depth_texture = RenderTexture::new(4096, 4096, true).unwrap();

    let ortho_projection = Mat4::orthographic_rh(-50.0, 50.0, -50.0, 50.0, 1.0, 100.0);

    // debug quad for depth texture
    let quad_shader = Shader::new_from_file(
        Path::new("./assets/shaders/debug_quad/vertex.glsl"),
        Path::new("./assets/shaders/debug_quad/fragment.glsl"),
    ).unwrap();

    let mut quad_mesh = Mesh::new();
    
    let quad_vertices = [
        // positions   // texture coords
        -1.0, -1.0,   0.0, 0.0,  // bottom-left
        -0.25, -1.0,   1.0, 0.0,  // bottom-right
        -0.25, -0.25,   1.0, 1.0,  // top-right
        -1.0, -0.25,   0.0, 1.0   // top-left
    ];

    let quad_indices = [0, 1, 3, 1, 2, 3];

    quad_mesh.update_vertices(&quad_vertices);
    quad_mesh.update_indices(&quad_indices);
    quad_mesh.add_vertex_attributes(&[
        (0, 2, gl::FLOAT, false),  // position
        (1, 2, gl::FLOAT, false)   // texture coord
    ]);

    while !window.should_window_close() {
        let vp = camera_controller.get_vp();
        camera_controller.update(&mut window);

        // Process one chunk per frame
        terrain_manager.process_chunk_generation();

        if window.is_key_pressed(WindowKey::Num1) {
            window.set_rendering_type(ferrousgl::RenderingType::Wireframe);
        } else if window.is_key_pressed(WindowKey::Num2) {
            window.set_rendering_type(ferrousgl::RenderingType::Solid);
        }

        if window.is_mouse_button_pressed(glfw::MouseButton::Left) {
            //terrain_manager.place_sphere(camera_controller.position, 2.0);
            terrain_manager.modify_terrain_with_raycast(camera_controller.get_ray(), 100.0, 5.0, false);
        }
        if window.is_mouse_button_pressed(glfw::MouseButton::Left) {
            //terrain_manager.place_voxel(camera_controller.position);
        }

        // begin rendering
        let light_pos = camera_controller.position + Vec3::new(0.0, 10.0, 0.0);
        let light_target = camera_controller.target;
        let light_up = Vec3::new(0.0, -1.0, 0.0);
        let light_view = Mat4::look_at_rh(light_pos, light_target, light_up);

        depth_texture.bind();
        depth_shader.bind_program();
        depth_shader.set_uniform_matrix_4fv("lightSpaceMatrix", 
        (ortho_projection * light_view).to_cols_array().as_ref());

        window.clear_color(Vec4::new(0.4, 0.4, 0.9, 1.0));
        window.clear_depth();
        window.set_depth_testing(DepthType::LessOrEqual);

        for (_, chunk) in &terrain_manager.chunks {
            if chunk.is_empty {
                continue;
            }

            let model = Mat4::from_translation(
                Vec3::new(
                    chunk.position.x as f32 * terrain_manager.chunk_size as f32,
                    chunk.position.y as f32 * terrain_manager.chunk_size as f32,
                    chunk.position.z as f32 * terrain_manager.chunk_size as f32
                ));
            depth_shader.set_uniform_matrix_4fv("model", model.to_cols_array().as_ref());

            window.render_mesh(chunk.get_mesh());
        }

        depth_shader.unbind_program();
        depth_texture.unbind();

        window.update_viewport(window.get_window_size().0, window.get_window_size().1);
        
        window.clear_color(Vec4::new(1.0, 1.0, 1.0, 1.0));
        window.clear_depth();
        window.set_depth_testing(DepthType::LessOrEqual);
        
        // begin actual color rendering

        terrain_manager.textures[0].bind(0); // Bind the first texture to texture unit 0
        terrain_manager.textures[1].bind(1);
        terrain_manager.textures[2].bind(2);
        terrain_manager.textures[3].bind(3);
        depth_texture.depth_texture().unwrap().bind(4);

        terrain_manager.terrain_shader.bind_program();
        terrain_manager.terrain_shader.set_uniform_3f("uLightDir", -0.8, 1.0, 0.5);
        terrain_manager.terrain_shader.set_uniform_texture("uGrassTex", 0);
        terrain_manager.terrain_shader.set_uniform_texture("uGrassNormal", 1);
        terrain_manager.terrain_shader.set_uniform_texture("uRockTex", 2);
        terrain_manager.terrain_shader.set_uniform_texture("uRockNormal",3);
        terrain_manager.terrain_shader.set_uniform_texture("shadowMap", 4);
        terrain_manager.terrain_shader.set_uniform_3f("lightPos", light_pos.x, light_pos.y, light_pos.z);
        terrain_manager.terrain_shader.set_uniform_3f("viewPos", camera_controller.position.x, camera_controller.position.y, camera_controller.position.z);
        terrain_manager.terrain_shader.set_uniform_1i("shadowBlurKernelSize", 1);
        terrain_manager.terrain_shader.set_uniform_3f("lightColor", 1.0, 1.0, 1.0);
        terrain_manager.terrain_shader.set_uniform_3f("ambientColor", 0.8, 0.85, 0.95);
        terrain_manager.terrain_shader.set_uniform_matrix_4fv("lightSpaceMatrix", 
            (ortho_projection * light_view).to_cols_array().as_ref());


        for (_, chunk) in &terrain_manager.chunks {
            if chunk.is_empty {
                continue;
            }

            let model = Mat4::from_translation(
                Vec3::new(
                    chunk.position.x as f32 * terrain_manager.chunk_size as f32,
                    chunk.position.y as f32 * terrain_manager.chunk_size as f32,
                    chunk.position.z as f32 * terrain_manager.chunk_size as f32
                ));
            let mvp = vp * model;
            terrain_manager.terrain_shader.set_uniform_matrix_4fv("uMVP", &mvp.to_cols_array());

            window.render_mesh(chunk.get_mesh());
        }

        terrain_manager.terrain_shader.unbind_program();

        // debug quad

        quad_shader.bind_program();
        depth_texture.depth_texture().unwrap().bind(0);
        quad_shader.set_uniform_texture("screenTexture", 0);
        window.set_depth_testing(DepthType::None);
        window.render_mesh(&quad_mesh);
        depth_texture.depth_texture().unwrap().unbind();

        // end rendering

        let title = format!(
            "Fallendust - FPS: {:.2} - Frame Time: {:.2}ms - Camera Position: {:?} - Active chunks: {:?} - Empty chunks: {:?} - Renderer: {:?}",
            1.0 / (window.get_frame_time() / 1_000_000.0),
            window.get_frame_time(),
            camera_controller.position,
            terrain_manager.get_active_chunks_count(),
            terrain_manager.get_empty_active_chunks_count(),
            unsafe { window.get_renderer() }
        );
        window.set_window_title(&title);
        window.update();
    }
}
