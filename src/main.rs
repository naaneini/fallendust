use std::{io::Write, path::Path};

use camera_controller::CameraController;
use ferrousgl::{DepthType, GlWindow, Mesh, MipmapType, RenderTexture, RenderingType, Shader, WindowConfig, WindowKey};
use glam::{vec3, IVec3, Mat4, Vec3, Vec4};
use terrain::terrain_manager::{self, TerrainManager};

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

    terrain_manager.enqueue_chunks_in_radius(IVec3::new(0, 0, 0), 4);
    
    // shadow stuff
    let depth_shader = Shader::new_from_file(
        Path::new("./assets/shaders/shadows/vertex.glsl"),
        Path::new("./assets/shaders/shadows/fragment.glsl"),
    ).unwrap();

    let mut depth_texture = RenderTexture::new(8192, 8192, true).unwrap();
    depth_texture.texture().bind(0);
    depth_texture.set_mipmap_type(MipmapType::Nearest);

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

    // typing
    let mut typing_command = false;
    let mut typed_keys: Vec<char> = Vec::new();

    while !window.should_window_close() {
        fn parse_and_execute_command(command: &str, terrain_manager: &mut TerrainManager) -> Result<(), String> {
            let parts: Vec<&str> = command.split_whitespace().collect();
            
            // Check command name and argument count
            if parts.is_empty() || parts[0] != "/p" {
            return Err("Command must start with '/p'".to_string());
            }
            
            if parts.len() != 8 {
            return Err(format!("Expected 7 arguments (got {}). Usage: place_voxel x y z size_x size_y size_z value", parts.len() - 1));
            }
            
            // Parse all numeric arguments
            let x = parts[1].parse::<i16>().map_err(|e| e.to_string())?;
            let y = parts[2].parse::<i16>().map_err(|e| e.to_string())?;
            let z = parts[3].parse::<i16>().map_err(|e| e.to_string())?;
            let size_x = parts[4].parse::<i16>().map_err(|e| e.to_string())?;
            let size_y = parts[5].parse::<i16>().map_err(|e| e.to_string())?;
            let size_z = parts[6].parse::<i16>().map_err(|e| e.to_string())?;
            let value = parts[7].parse::<f32>().map_err(|e| e.to_string())?;
            
            // Execute the function
            terrain_manager.place_voxel_in_chunk(
            IVec3::new(x as i32, y as i32, z as i32),
            IVec3::new(size_x as i32, size_y as i32, size_z as i32),
            value,
            );
            
            Ok(())
        }

        if window.is_key_pressed(WindowKey::Slash) {
            typing_command = true;
            println!("Enter CMD:");
        }
        if window.is_key_pressed(WindowKey::Enter) {
            typing_command = false;
            let command = typed_keys.iter().collect::<String>();
            println!("Typed command: {:?}", command);
            
            match parse_and_execute_command(&command, &mut terrain_manager) {
            Ok(_) => println!("Command executed successfully"),
            Err(e) => println!("Error parsing command: {}", e),
            }
            
            typed_keys.clear();
        }
        if typing_command {
            window.get_typed_keys().iter().for_each(|key| {
            typed_keys.push(*key);
            print!("{}", *key);
            std::io::stdout().flush().unwrap();
            });
        }

        let vp = camera_controller.get_vp();
        camera_controller.update(&mut window);

        // Process one chunk per frame
        terrain_manager.process_chunk_generation();

        if window.is_key_pressed(WindowKey::F1) {
            window.set_rendering_type(ferrousgl::RenderingType::Wireframe);
        } else if window.is_key_pressed(WindowKey::F2) {
            window.set_rendering_type(ferrousgl::RenderingType::Solid);
        }

        if window.is_mouse_button_pressed(glfw::MouseButton::Left) {
            let ray = camera_controller.get_ray();
            terrain_manager.create_sphere(camera_controller.position, 4.0);
            
            if let Some(hit_position) = terrain_manager.raycast(&ray, 1000.0) {
                //terrain_manager.create_sphere(hit_position, 2.0);
            } else {
                // Handle case when raycast doesn't hit anything
                println!("Raycast didn't hit any terrain");
            }
        }
        if window.is_mouse_button_pressed(glfw::MouseButton::Left) {
            //terrain_manager.place_voxel(camera_controller.position);
        }

        // begin rendering
        let ortho_projection = Mat4::orthographic_rh(-128.0, 128.0, -128.0, 128.0, 0.0, 155.0);

        // Light comes from above (higher Y value) but moves with camera
        let light_height = 10.0; // How high above the camera the light is
        let light_offset = Vec3::new(0.0, light_height, 0.0); // Directly above
        let light_pos = camera_controller.position + light_offset;

        // Light points downward (toward camera position)
        let light_target = camera_controller.position; 
        let light_view = Mat4::look_at_rh(light_pos, light_target, Vec3::Z); // Using Z as up vector for a vertical light
        let light_dir = (light_target - light_pos).normalize();

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
        
        window.clear_color(Vec4::new(0.4, 0.4, 0.9, 1.0));
        window.clear_depth();
        //window.set_depth_testing(DepthType::LessOrEqual);
        
        // begin actual color rendering

        terrain_manager.terrain_shader.bind_program();
        terrain_manager.terrain_shader.set_uniform_3f("lightPos", 0.0, 0.0, 0.0);
        terrain_manager.terrain_shader.set_uniform_3f("viewPos", camera_controller.position.x, camera_controller.position.y, camera_controller.position.z);
        terrain_manager.terrain_shader.set_uniform_3f("lightDir", light_dir.x, light_dir.y, light_dir.z);
        terrain_manager.terrain_shader.set_uniform_matrix_4fv("lightSpaceMatrix", 
            (ortho_projection * light_view).to_cols_array().as_ref());

        // Set projection and view matrices
        terrain_manager.terrain_shader.set_uniform_matrix_4fv("projection", &camera_controller.get_projection().to_cols_array());
        terrain_manager.terrain_shader.set_uniform_matrix_4fv("view", &camera_controller.get_view().to_cols_array());

        terrain_manager.textures[0].bind(0); // Bind the first texture to texture unit 0
        terrain_manager.textures[1].bind(1);
        terrain_manager.textures[2].bind(2);
        terrain_manager.textures[3].bind(3);
        depth_texture.depth_texture().unwrap().bind(4);
        terrain_manager.terrain_shader.set_uniform_texture("uGrassTex", 0);
        terrain_manager.terrain_shader.set_uniform_texture("uGrassNormal", 1);
        terrain_manager.terrain_shader.set_uniform_texture("uRockTex", 2);
        terrain_manager.terrain_shader.set_uniform_texture("uRockNormal", 3);
        terrain_manager.terrain_shader.set_uniform_texture("shadowMap", 4);

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
            terrain_manager.terrain_shader.set_uniform_matrix_4fv("model", &model.to_cols_array());
                
            //window.set_rendering_type(RenderingType::Solid);
            window.render_mesh(chunk.get_mesh());
            //window.set_rendering_type(RenderingType::Wireframe);
            //window.render_mesh(chunk.get_bounding_box_mesh());
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
            "EngineCore Fallendust x64 - FPS: {:.2} - FT: {:.2}ms - camPos: {:?} - RNDR: {:?} [DEBUG F1, F2, F3]",
            1.0 / (window.get_frame_time() / 1_000_000.0),
            window.get_frame_time(),
            camera_controller.position,
            unsafe { window.get_renderer() }
        );
        window.set_window_title(&title);
        window.update();
    }
}
