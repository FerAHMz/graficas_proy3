use crate::framebuffer::Framebuffer;
use crate::color::Color;

pub struct Skybox {
    stars: Vec<Star>,
}

struct Star {
    x: f32,
    y: f32,
    brightness: u8,
}

impl Skybox {
    pub fn new(star_count: usize) -> Self {
        let mut stars = Vec::new();
        
        // Generar estrellas proceduralmente usando un seed simple
        let mut seed = 12345u32;
        
        for _ in 0..star_count {
            // Generador simple de números pseudo-aleatorios
            seed = seed.wrapping_mul(1103515245).wrapping_add(12345);
            let rand1 = (seed >> 16) as f32 / 65536.0;
            
            seed = seed.wrapping_mul(1103515245).wrapping_add(12345);
            let rand2 = (seed >> 16) as f32 / 65536.0;
            
            seed = seed.wrapping_mul(1103515245).wrapping_add(12345);
            let rand3 = (seed >> 16) as f32 / 65536.0;
            
            // Posiciones normalizadas entre 0 y 1
            let x = rand1;
            let y = rand2;
            
            // Brillo variado (más estrellas tenues que brillantes)
            let brightness = if rand3 > 0.9 {
                255 // Estrellas muy brillantes (10%)
            } else if rand3 > 0.7 {
                200 // Estrellas brillantes (20%)
            } else if rand3 > 0.4 {
                150 // Estrellas medias (30%)
            } else {
                100 // Estrellas tenues (40%)
            };
            
            stars.push(Star { x, y, brightness });
        }
        
        Skybox { stars }
    }
    
    pub fn render(&self, framebuffer: &mut Framebuffer) {
        let width = framebuffer.width as f32;
        let height = framebuffer.height as f32;
        
        for star in &self.stars {
            let screen_x = (star.x * width) as usize;
            let screen_y = (star.y * height) as usize;
            
            if screen_x < framebuffer.width && screen_y < framebuffer.height {
                let color = Color::new(star.brightness, star.brightness, star.brightness);
                
                // Dibujar estrella (punto simple)
                framebuffer.set_current_color(color.to_hex());
                framebuffer.point(screen_x, screen_y, 1.0);
                
                // Estrellas brillantes tienen un halo pequeño
                if star.brightness > 200 {
                    let dimmer = (star.brightness as f32 * 0.5) as u8;
                    let halo_color = Color::new(dimmer, dimmer, dimmer);
                    framebuffer.set_current_color(halo_color.to_hex());
                    
                    // Píxeles adyacentes para el halo
                    if screen_x > 0 {
                        framebuffer.point(screen_x - 1, screen_y, 1.0);
                    }
                    if screen_x < framebuffer.width - 1 {
                        framebuffer.point(screen_x + 1, screen_y, 1.0);
                    }
                    if screen_y > 0 {
                        framebuffer.point(screen_x, screen_y - 1, 1.0);
                    }
                    if screen_y < framebuffer.height - 1 {
                        framebuffer.point(screen_x, screen_y + 1, 1.0);
                    }
                }
            }
        }
    }
}
