use nalgebra_glm::{Vec2, Vec3, Vec4, Mat4, look_at, perspective};
use minifb::{Key, Window, WindowOptions};
use std::time::Duration;
use std::f32::consts::PI;

mod framebuffer;
mod triangle;
mod vertex;
mod obj;
mod color;
mod fragment;
mod shaders;
mod sphere;
mod planets;
mod skybox;
mod line;

use framebuffer::Framebuffer;
use vertex::Vertex;
use obj::Obj;
use triangle::{triangle, triangle_with_shader};
use shaders::{vertex_shader, Uniforms};
use planets::{Planet, PlanetType, Moon, Ring};
use skybox::Skybox;
use color::Color;
use line::line;

// Estructura para la nave espacial
struct Spaceship {
    position: Vec3,
    rotation: Vec3,  // (pitch, yaw, roll)
    scale: f32,
    speed: f32,
    rotation_speed: f32,
}

impl Spaceship {
    fn check_collision(&self, planet_pos: Vec3, planet_radius: f32) -> bool {
        let distance = ((self.position.x - planet_pos.x).powi(2) +
                       (self.position.y - planet_pos.y).powi(2) +
                       (self.position.z - planet_pos.z).powi(2)).sqrt();
        
        // Radio de colisión de la nave (más grande para evitar acercarse demasiado)
        let spaceship_radius = 1.5;
        // Agregar margen de seguridad extra
        let safety_margin = 1.0;
        distance < (planet_radius + spaceship_radius + safety_margin)
    }
}

fn create_model_matrix(translation: Vec3, scale: f32, rotation: Vec3) -> Mat4 {
    let (sin_x, cos_x) = rotation.x.sin_cos();
    let (sin_y, cos_y) = rotation.y.sin_cos();
    let (sin_z, cos_z) = rotation.z.sin_cos();

    let rotation_matrix_x = Mat4::new(
        1.0,  0.0,    0.0,   0.0,
        0.0,  cos_x, -sin_x, 0.0,
        0.0,  sin_x,  cos_x, 0.0,
        0.0,  0.0,    0.0,   1.0,
    );

    let rotation_matrix_y = Mat4::new(
        cos_y,  0.0,  sin_y, 0.0,
        0.0,    1.0,  0.0,   0.0,
        -sin_y, 0.0,  cos_y, 0.0,
        0.0,    0.0,  0.0,   1.0,
    );

    let rotation_matrix_z = Mat4::new(
        cos_z, -sin_z, 0.0, 0.0,
        sin_z,  cos_z, 0.0, 0.0,
        0.0,    0.0,  1.0, 0.0,
        0.0,    0.0,  0.0, 1.0,
    );

    let rotation_matrix = rotation_matrix_z * rotation_matrix_y * rotation_matrix_x;

    let transform_matrix = Mat4::new(
        scale, 0.0,   0.0,   translation.x,
        0.0,   scale, 0.0,   translation.y,
        0.0,   0.0,   scale, translation.z,
        0.0,   0.0,   0.0,   1.0,
    );

    transform_matrix * rotation_matrix
}

fn create_view_matrix(eye: Vec3, center: Vec3, up: Vec3) -> Mat4 {
    look_at(&eye, &center, &up)
}

fn create_perspective_matrix(window_width: f32, window_height: f32) -> Mat4 {
    let fov = 45.0 * PI / 180.0;
    let aspect_ratio = window_width / window_height;
    let near = 1.0;  // Aumentado a 1.0 para evitar ver el interior de planetas
    let far = 100.0;

    perspective(fov, aspect_ratio, near, far)
}

fn create_viewport_matrix(width: f32, height: f32) -> Mat4 {
    Mat4::new(
        width / 2.0, 0.0, 0.0, width / 2.0,
        0.0, -height / 2.0, 0.0, height / 2.0,
        0.0, 0.0, 1.0, 0.0,
        0.0, 0.0, 0.0, 1.0
    )
}

fn draw_orbit(
    framebuffer: &mut Framebuffer,
    center: Vec3,
    radius: f32,
    segments: u32,
    color: Color,
    view_matrix: &Mat4,
    projection_matrix: &Mat4,
    viewport_matrix: &Mat4,
) {
    let mvp = projection_matrix * view_matrix;
    
    for i in 0..segments {
        let angle1 = (i as f32 / segments as f32) * 2.0 * PI;
        let angle2 = ((i + 1) as f32 / segments as f32) * 2.0 * PI;
        
        let p1 = Vec3::new(
            center.x + radius * angle1.cos(),
            center.y,
            center.z + radius * angle1.sin()
        );
        let p2 = Vec3::new(
            center.x + radius * angle2.cos(),
            center.y,
            center.z + radius * angle2.sin()
        );
        
        let p1_clip = mvp * Vec4::new(p1.x, p1.y, p1.z, 1.0);
        let p2_clip = mvp * Vec4::new(p2.x, p2.y, p2.z, 1.0);
        
        if p1_clip.w.abs() < 0.001 || p2_clip.w.abs() < 0.001 {
            continue;
        }
        
        let p1_ndc = Vec3::new(
            p1_clip.x / p1_clip.w,
            p1_clip.y / p1_clip.w,
            p1_clip.z / p1_clip.w
        );
        let p2_ndc = Vec3::new(
            p2_clip.x / p2_clip.w,
            p2_clip.y / p2_clip.w,
            p2_clip.z / p2_clip.w
        );
        
        if p1_ndc.x.abs() > 2.0 || p1_ndc.y.abs() > 2.0 || 
           p2_ndc.x.abs() > 2.0 || p2_ndc.y.abs() > 2.0 ||
           p1_clip.w < 0.0 || p2_clip.w < 0.0 {
            continue;
        }
        
        let viewport = viewport_matrix;
        let p1_screen = viewport * Vec4::new(p1_ndc.x, p1_ndc.y, p1_ndc.z, 1.0);
        let p2_screen = viewport * Vec4::new(p2_ndc.x, p2_ndc.y, p2_ndc.z, 1.0);
        
        if p1_screen.x.is_nan() || p1_screen.y.is_nan() || 
           p2_screen.x.is_nan() || p2_screen.y.is_nan() {
            continue;
        }
        
        let v1 = Vertex {
            position: p1,
            normal: Vec3::new(0.0, 1.0, 0.0),
            tex_coords: Vec2::new(0.0, 0.0),
            color: color.clone(),
            transformed_position: Vec3::new(p1_screen.x, p1_screen.y, p1_screen.z),
            transformed_normal: Vec3::new(0.0, 1.0, 0.0),
        };
        let v2 = Vertex {
            position: p2,
            normal: Vec3::new(0.0, 1.0, 0.0),
            tex_coords: Vec2::new(0.0, 0.0),
            color: color.clone(),
            transformed_position: Vec3::new(p2_screen.x, p2_screen.y, p2_screen.z),
            transformed_normal: Vec3::new(0.0, 1.0, 0.0),
        };
        
        line(&v1, &v2, framebuffer);
    }
}

fn render(framebuffer: &mut Framebuffer, uniforms: &Uniforms, vertex_array: &[Vertex]) {
    render_with_shader(framebuffer, uniforms, vertex_array, None, 0.0)
}

fn render_with_shader(framebuffer: &mut Framebuffer, uniforms: &Uniforms, vertex_array: &[Vertex], shader_type: Option<PlanetType>, time: f32) {
    // Vertex Shader Stage
    let mut transformed_vertices = Vec::with_capacity(vertex_array.len());
    for vertex in vertex_array {
        let transformed = vertex_shader(vertex, uniforms);
        transformed_vertices.push(transformed);
    }

    // Primitive Assembly Stage - manually iterate through faces
    let mut triangles = Vec::new();
    for i in (0..transformed_vertices.len()).step_by(3) {
        if i + 2 < transformed_vertices.len() {
            triangles.push([
                transformed_vertices[i].clone(),
                transformed_vertices[i + 1].clone(),
                transformed_vertices[i + 2].clone(),
            ]);
        }
    }

    // Rasterization Stage - draw all triangles
    let mut fragments = Vec::new();
    for tri in &triangles {
        fragments.extend(triangle_with_shader(&tri[0], &tri[1], &tri[2], shader_type.clone(), time));
    }

    // Fragment Processing Stage
    for fragment in fragments {
        let x = fragment.position.x as usize;
        let y = fragment.position.y as usize;
        if x < framebuffer.width && y < framebuffer.height {
            let color = fragment.color.to_hex();
            framebuffer.set_current_color(color);
            framebuffer.point(x, y, fragment.depth);
        }
    }
}

fn handle_spaceship_controls(window: &Window, spaceship: &mut Spaceship) {
    let yaw = spaceship.rotation.y;
    let pitch = spaceship.rotation.x;
    
    // Movimiento adelante/atrás basado en la dirección YAW
    if window.is_key_down(Key::W) {
        spaceship.position.x += yaw.sin() * spaceship.speed;
        spaceship.position.z += yaw.cos() * spaceship.speed;
    }
    if window.is_key_down(Key::S) {
        spaceship.position.x -= yaw.sin() * spaceship.speed;
        spaceship.position.z -= yaw.cos() * spaceship.speed;
    }
    
    // Rotación horizontal (YAW)
    if window.is_key_down(Key::A) {
        spaceship.rotation.y += spaceship.rotation_speed;
    }
    if window.is_key_down(Key::D) {
        spaceship.rotation.y -= spaceship.rotation_speed;
    }
    
    // Inclinación (PITCH) - Limitado a ±30 grados
    if window.is_key_down(Key::Up) {
        spaceship.rotation.x += spaceship.rotation_speed * 0.5;
        spaceship.rotation.x = spaceship.rotation.x.min(PI / 6.0);  // Max 30°
    }
    if window.is_key_down(Key::Down) {
        spaceship.rotation.x -= spaceship.rotation_speed * 0.5;
        spaceship.rotation.x = spaceship.rotation.x.max(-PI / 6.0); // Min -30°
    }
    
    // Movimiento vertical
    if window.is_key_down(Key::Q) {
        spaceship.position.y += spaceship.speed;
    }
    if window.is_key_down(Key::E) {
        spaceship.position.y -= spaceship.speed;
    }
}

fn main() {
    let window_width = 800;
    let window_height = 800;
    let framebuffer_width = 800;
    let framebuffer_height = 800;
    let frame_delay = Duration::from_millis(16);

    let mut framebuffer = Framebuffer::new(framebuffer_width, framebuffer_height);
    let mut window = Window::new(
        "Solar System Renderer - Camera Following Spaceship",
        window_width,
        window_height,
        WindowOptions::default(),
    )
    .unwrap();

    window.set_position(500, 500);
    window.update();

    // Set background color to deep space
    framebuffer.set_background_color(0x000000);
    
    // Crear skybox con estrellas
    let skybox = Skybox::new(800);

    // Cargar modelo de la nave
    let spaceship_obj = Obj::load("assets/Spaceship.obj").expect("Error cargando modelo de nave");
    let spaceship_vertices = spaceship_obj.get_vertex_array();

    // Inicializar la nave espacial
    let mut spaceship = Spaceship {
        position: Vec3::new(0.0, 8.0, 35.0),  // Igual que el proyecto de referencia
        rotation: Vec3::new(0.0, PI, 0.0),    // Mirando hacia -Z
        scale: 0.08,  // Escala pequeña como en referencia
        speed: 0.15,  // Velocidad de referencia
        rotation_speed: 0.03,  // Velocidad de rotación de referencia
    };

    // Create planetary system for maximum points
    let mut planets = Vec::new();
    let mut moons = Vec::new();
    let mut rings = Vec::new();

    // Sun (Star) - Center of the system (ahora en coordenadas 3D)
    planets.push(Planet::new(
        PlanetType::Star,
        Vec3::new(0.0, 0.0, 0.0), // Centro del sistema en 3D
        2.0, // Escala del sol (como referencia)
        0.5,  // Slow rotation
        0.0,  // No orbital motion (it's the center)
        0.0,
    ));

    // Rocky Planet (Earth-like) with moon
    planets.push(Planet::new(
        PlanetType::RockyPlanet,
        Vec3::new(0.0, 0.0, 0.0),
        0.8,  // Escala reducida
        2.0,  // Rotation
        0.5,  // Orbital speed
        8.0,  // Orbital radius (como referencia)
    ));

    // Moon for the rocky planet
    moons.push(Moon::new(
        Vec3::new(0.0, 0.0, 0.0), // Will be updated
        1.5,  // Orbital radius around planet (como referencia)
        2.0,  // Fast orbital speed
        0.3,  // Small size (como referencia)
    ));

    // Gas Giant with rings
    planets.push(Planet::new(
        PlanetType::GasGiant,
        Vec3::new(0.0, 0.0, 0.0),
        1.5,  // Escala reducida
        1.5,
        0.3,  // Orbital speed
        15.0, // Orbital radius (como referencia)
    ));

    // Rings for gas giant - proper spacing from planet surface
    rings.push(Ring::new(2.0, 2.5, 128)); // Main ring system 
    rings.push(Ring::new(2.6, 3.0, 96));  // Outer ring
    rings.push(Ring::new(1.8, 1.95, 96)); // Inner ring

    // Extra planets for bonus points
    // Ice Planet
    planets.push(Planet::new(
        PlanetType::IcePlanet,
        Vec3::new(0.0, 0.0, 0.0),
        0.6,  // Escala reducida
        1.0,
        0.2,  // Orbital speed
        22.0, // Orbital radius (como referencia)
    ));

    // Volcanic Planet
    planets.push(Planet::new(
        PlanetType::VolcanicPlanet,
        Vec3::new(0.0, 0.0, 0.0),
        0.5,  // Escala reducida
        3.0,
        1.5,
        5.0,  // Close to the sun (reducido)
    ));

    // Ringed Planet (Saturn-like)
    planets.push(Planet::new(
        PlanetType::RingedPlanet,
        Vec3::new(0.0, 0.0, 0.0),
        1.2,  // Escala reducida
        1.2,
        0.4,
        28.0, // Orbital radius (reducido)
    ));

    // Rings for ringed planet - Saturn-like with proper spacing  
    rings.push(Ring::new(1.7, 2.2, 128));  // Main A ring
    rings.push(Ring::new(2.3, 2.8, 96));   // B ring
    rings.push(Ring::new(1.5, 1.65, 64));  // Inner C ring

    let start_time = std::time::Instant::now();
    
    println!("🌟 SOLAR SYSTEM RENDERER 🌟");
    println!("=====================================");
    println!("Features implemented:");
    println!("✓ Camera following spaceship in 3rd person");
    println!("✓ Star (Sun) - 4-layer fire shader");
    println!("✓ Rocky Planet - 4-layer Earth-like shader");
    println!("✓ Gas Giant - 4-layer Jupiter-like shader");
    println!("✓ Ice Planet - 4-layer frozen world shader"); 
    println!("✓ Volcanic Planet - 4-layer lava world shader");
    println!("✓ Ringed Planet - 4-layer Saturn-like shader");
    println!("✓ Moon system - orbiting rocky planet");
    println!("✓ Ring systems - around gas giants");
    println!("=====================================");
    println!("SPACESHIP CONTROLS:");
    println!("• W/S: Forward/Backward");
    println!("• A/D: Rotate left/right");
    println!("• Arrow Up/Down: Pitch up/down");
    println!("• Q/E: Move up/down");
    println!("• ESC: Exit");
    println!("=====================================");

    // Crear matrices de proyección y viewport
    let projection_matrix = create_perspective_matrix(window_width as f32, window_height as f32);
    let viewport_matrix = create_viewport_matrix(framebuffer_width as f32, framebuffer_height as f32);

    while window.is_open() {
        if window.is_key_down(Key::Escape) {
            break;
        }

        let elapsed = start_time.elapsed().as_secs_f32();

        // Guardar posición anterior para detectar colisiones
        let old_position = spaceship.position;

        // Manejar controles de la nave
        handle_spaceship_controls(&window, &mut spaceship);

        // Detectar colisiones con planetas
        let mut collision_detected = false;
        for planet in &planets {
            let planet_pos = planet.get_current_position();
            if spaceship.check_collision(planet_pos, planet.scale) {
                collision_detected = true;
                break;
            }
        }
        
        // Detectar colisiones con lunas
        if !moons.is_empty() && planets.len() > 1 {
            let moon_pos = moons[0].get_current_position();
            if spaceship.check_collision(moon_pos, moons[0].scale) {
                collision_detected = true;
            }
        }
        
        // Si hay colisión, revertir movimiento
        if collision_detected {
            spaceship.position = old_position;
        }

        // Update planetary positions
        let delta_time = 0.016; // Assuming ~60 FPS
        for planet in &mut planets {
            planet.update(delta_time);
        }

        // Update moon positions
        if planets.len() > 1 { // Make sure rocky planet exists
            let rocky_planet_pos = planets[1].get_current_position();
            for moon in &mut moons {
                moon.update(delta_time, rocky_planet_pos);
            }
        }

        // CÁMARA EN TERCERA PERSONA SIGUIENDO LA NAVE
        let camera_distance = 2.5;   // Distancia de referencia
        let camera_height = 0.8;     // Altura de referencia
        let yaw = spaceship.rotation.y;
        
        // Calcular posición de cámara DETRÁS de la nave
        let camera_position = Vec3::new(
            spaceship.position.x - yaw.sin() * camera_distance,
            spaceship.position.y + camera_height,
            spaceship.position.z - yaw.cos() * camera_distance
        );
        
        // Crear view matrix: cámara mira hacia la nave
        let view_matrix = create_view_matrix(
            camera_position,
            spaceship.position,
            Vec3::new(0.0, 1.0, 0.0)  // Vector UP
        );

        framebuffer.clear();
        
        // Renderizar skybox con estrellas
        skybox.render(&mut framebuffer);
        
        // Dibujar órbitas de los planetas
        draw_orbit(
            &mut framebuffer,
            Vec3::new(0.0, 0.0, 0.0),
            8.0,
            100,
            Color::new(0, 255, 100),
            &view_matrix,
            &projection_matrix,
            &viewport_matrix
        );
        
        draw_orbit(
            &mut framebuffer,
            Vec3::new(0.0, 0.0, 0.0),
            15.0,
            120,
            Color::new(200, 100, 255),
            &view_matrix,
            &projection_matrix,
            &viewport_matrix
        );
        
        draw_orbit(
            &mut framebuffer,
            Vec3::new(0.0, 0.0, 0.0),
            22.0,
            140,
            Color::new(100, 200, 255),
            &view_matrix,
            &projection_matrix,
            &viewport_matrix
        );
        
        // Órbita del planeta anillado (más externa)
        draw_orbit(
            &mut framebuffer,
            Vec3::new(0.0, 0.0, 0.0),
            28.0,
            160,
            Color::new(150, 255, 150),
            &view_matrix,
            &projection_matrix,
            &viewport_matrix
        );

        // Render all planets
        for (i, planet) in planets.iter().enumerate() {
            let translation = planet.get_current_position();
            let rotation = Vec3::new(0.0, planet.current_rotation, 0.0);
            let scale = planet.scale;

            let model_matrix = create_model_matrix(
                translation,
                scale,
                rotation
            );
            let uniforms = Uniforms { 
                model_matrix,
                view_matrix,
                projection_matrix,
                viewport_matrix,
            };

            render_with_shader(
                &mut framebuffer,
                &uniforms,
                planet.sphere.get_vertex_array(),
                Some(planet.planet_type.clone()),
                elapsed
            );

            // Render rings if this is a gas giant or ringed planet
            if matches!(planet.planet_type, PlanetType::GasGiant) && i == 2 && rings.len() >= 3 {
                // Render multiple rings for gas giant
                render_ring(&mut framebuffer, &rings[0], translation, planet.scale, elapsed, &view_matrix, &projection_matrix, &viewport_matrix);
                render_ring(&mut framebuffer, &rings[1], translation, planet.scale, elapsed, &view_matrix, &projection_matrix, &viewport_matrix);
                render_ring(&mut framebuffer, &rings[2], translation, planet.scale, elapsed, &view_matrix, &projection_matrix, &viewport_matrix);
            }
            if matches!(planet.planet_type, PlanetType::RingedPlanet) && rings.len() >= 6 {
                // Render multiple rings for ringed planet (Saturn-like)
                render_ring(&mut framebuffer, &rings[3], translation, planet.scale, elapsed, &view_matrix, &projection_matrix, &viewport_matrix);
                render_ring(&mut framebuffer, &rings[4], translation, planet.scale, elapsed, &view_matrix, &projection_matrix, &viewport_matrix);
                render_ring(&mut framebuffer, &rings[5], translation, planet.scale, elapsed, &view_matrix, &projection_matrix, &viewport_matrix);
            }
        }

        // Render moons
        if !moons.is_empty() && planets.len() > 1 {
            let moon_pos = moons[0].get_current_position();
            let model_matrix = create_model_matrix(
                moon_pos,
                moons[0].scale,
                Vec3::new(0.0, 0.0, 0.0)
            );
            let uniforms = Uniforms { 
                model_matrix,
                view_matrix,
                projection_matrix,
                viewport_matrix,
            };

            // Use a simple gray color for moon
            framebuffer.set_current_color(0xAAAAA0);
            render(&mut framebuffer, &uniforms, moons[0].sphere.get_vertex_array());
        }
        
        // RENDERIZAR LA NAVE
        // Corrección de orientación del modelo - Spaceship.obj necesita rotación diferente a Jett.obj
        let spaceship_corrected_rotation = Vec3::new(
            spaceship.rotation.x + PI,  // Flip de 180° en X
            spaceship.rotation.y - PI/2.0,  // Rotación de -90° en Y para Spaceship.obj
            spaceship.rotation.z
        );
        
        let spaceship_model_matrix = create_model_matrix(
            spaceship.position,
            spaceship.scale,
            spaceship_corrected_rotation
        );
        let spaceship_uniforms = Uniforms {
            model_matrix: spaceship_model_matrix,
            view_matrix,
            projection_matrix,
            viewport_matrix,
        };
        render(&mut framebuffer, &spaceship_uniforms, &spaceship_vertices);

        window
            .update_with_buffer(&framebuffer.buffer, framebuffer_width, framebuffer_height)
            .unwrap();

        std::thread::sleep(frame_delay);
    }
}

fn render_ring(framebuffer: &mut Framebuffer, ring: &Ring, center: Vec3, scale: f32, time: f32, view_matrix: &Mat4, projection_matrix: &Mat4, viewport_matrix: &Mat4) {
    // Set ring color - make it more visible
    framebuffer.set_current_color(0xDDDDEE);
    
    // Create transformation matrix for the ring
    let ring_scale = scale * 0.012; // Adjust scale to be more visible
    let rotation = Vec3::new(75.0_f32.to_radians(), time * 0.2, 0.0); // Slight tilt and slow rotation
    let model_matrix = create_model_matrix(center, ring_scale, rotation);
    let uniforms = Uniforms { 
        model_matrix,
        view_matrix: *view_matrix,
        projection_matrix: *projection_matrix,
        viewport_matrix: *viewport_matrix,
    };
    
    // Render the ring
    render(framebuffer, &uniforms, &ring.vertices);
}
