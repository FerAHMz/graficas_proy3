use nalgebra_glm::{Vec3, dot};
use crate::fragment::Fragment;
use crate::vertex::Vertex;
use crate::color::Color;
use crate::planets::PlanetType;

pub fn triangle(v1: &Vertex, v2: &Vertex, v3: &Vertex) -> Vec<Fragment> {
    triangle_with_shader(v1, v2, v3, None, 0.0)
}

pub fn triangle_with_shader(v1: &Vertex, v2: &Vertex, v3: &Vertex, shader_type: Option<PlanetType>, time: f32) -> Vec<Fragment> {
    let mut fragments = Vec::new();
    let (a, b, c) = (v1.transformed_position, v2.transformed_position, v3.transformed_position);

    // Early rejection: skip if all vertices are NaN or Inf
    if !a.x.is_finite() || !a.y.is_finite() || !a.z.is_finite() ||
       !b.x.is_finite() || !b.y.is_finite() || !b.z.is_finite() ||
       !c.x.is_finite() || !c.y.is_finite() || !c.z.is_finite() {
        return fragments;
    }

    let (min_x, min_y, max_x, max_y) = calculate_bounding_box(&a, &b, &c);
    
    // Skip if bounding box is degenerate
    if min_x >= max_x || min_y >= max_y {
        return fragments;
    }

    let light_dir = Vec3::new(0.0, 0.0, -1.0);
    let triangle_area = edge_function(&a, &b, &c);
    
    // Skip if triangle has zero area
    if triangle_area.abs() < 0.001 {
        return fragments;
    }

    // Iterate over each pixel in the bounding box
    for y in min_y..=max_y {
        for x in min_x..=max_x {
            let point = Vec3::new(x as f32 + 0.5, y as f32 + 0.5, 0.0);

            // Calculate barycentric coordinates
            let (w1, w2, w3) = barycentric_coordinates(&point, &a, &b, &c, triangle_area);

            // Check if the point is inside the triangle
      if w1 >= 0.0 && w1 <= 1.0 && 
         w2 >= 0.0 && w2 <= 1.0 &&
         w3 >= 0.0 && w3 <= 1.0 {
        // Interpolate attributes
        let normal = (v1.transformed_normal * w1 + v2.transformed_normal * w2 + v3.transformed_normal * w3).normalize();
        let uv = v1.tex_coords * w1 + v2.tex_coords * w2 + v3.tex_coords * w3;
        let world_pos = v1.position * w1 + v2.position * w2 + v3.position * w3;

        let lit_color = if let Some(planet_type) = &shader_type {
          // Use planetary shader
          match planet_type {
            PlanetType::Star => crate::planets::star_shader(world_pos, normal, uv, time),
            PlanetType::RockyPlanet => crate::planets::rocky_planet_shader(world_pos, normal, uv, time),
            PlanetType::GasGiant => crate::planets::gas_giant_shader(world_pos, normal, uv, time),
            PlanetType::IcePlanet => crate::planets::ice_planet_shader(world_pos, normal, uv, time),
            PlanetType::VolcanicPlanet => crate::planets::volcanic_planet_shader(world_pos, normal, uv, time),
            PlanetType::RingedPlanet => crate::planets::ringed_planet_shader(world_pos, normal, uv, time),
          }
        } else {
          // Default lighting for spaceship
          let light_dir = Vec3::new(0.0, 0.0, -1.0);
          let intensity = dot(&normal, &light_dir).max(0.0);
          let base_color = Color::new(100, 150, 255);
          base_color * intensity.max(0.3)
        };

        let depth = a.z;                fragments.push(Fragment::new(x as f32, y as f32, lit_color, depth));
            }
        }
    }

    fragments
}

fn calculate_bounding_box(v1: &Vec3, v2: &Vec3, v3: &Vec3) -> (i32, i32, i32, i32) {
    let min_x = v1.x.min(v2.x).min(v3.x).floor() as i32;
    let min_y = v1.y.min(v2.y).min(v3.y).floor() as i32;
    let max_x = v1.x.max(v2.x).max(v3.x).ceil() as i32;
    let max_y = v1.y.max(v2.y).max(v3.y).ceil() as i32;

    // Clamp to reasonable screen bounds to prevent infinite loops
    let min_x = min_x.max(0).min(800);
    let min_y = min_y.max(0).min(600);
    let max_x = max_x.max(0).min(800);
    let max_y = max_y.max(0).min(600);

    (min_x, min_y, max_x, max_y)
}

fn barycentric_coordinates(p: &Vec3, a: &Vec3, b: &Vec3, c: &Vec3, area: f32) -> (f32, f32, f32) {
    let w1 = edge_function(b, c, p) / area;
    let w2 = edge_function(c, a, p) / area;
    let w3 = edge_function(a, b, p) / area;

    (w1, w2, w3)
}

fn edge_function(a: &Vec3, b: &Vec3, c: &Vec3) -> f32 {
    (c.x - a.x) * (b.y - a.y) - (c.y - a.y) * (b.x - a.x)
}
