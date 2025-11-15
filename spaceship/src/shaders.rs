use nalgebra_glm::{Vec3, Vec4, Mat4};
use crate::vertex::Vertex;

pub struct Uniforms {
    pub model_matrix: Mat4,
    pub view_matrix: Mat4,
    pub projection_matrix: Mat4,
    pub viewport_matrix: Mat4,
}

pub fn vertex_shader(vertex: &Vertex, uniforms: &Uniforms) -> Vertex {
    let position = Vec4::new(vertex.position.x, vertex.position.y, vertex.position.z, 1.0);
    
    // Apply transformations: Model -> View -> Projection
    let clip_position = uniforms.projection_matrix * uniforms.view_matrix * uniforms.model_matrix * position;
    
    // Perspective divide with safety check
    let ndc_position = if clip_position.w.abs() > 0.001 {
        Vec3::new(
            clip_position.x / clip_position.w,
            clip_position.y / clip_position.w,
            clip_position.z / clip_position.w,
        )
    } else {
        // Skip vertices that are too close or behind camera
        Vec3::new(0.0, 0.0, -1.0)
    };
    
    // Apply viewport transformation
    let screen_position = uniforms.viewport_matrix * Vec4::new(ndc_position.x, ndc_position.y, ndc_position.z, 1.0);
    let transformed_pos = Vec3::new(screen_position.x, screen_position.y, screen_position.z);
    
    // Transform normal (only model transformation, no translation)
    let normal4 = Vec4::new(vertex.normal.x, vertex.normal.y, vertex.normal.z, 0.0);
    let transformed_normal = uniforms.model_matrix * normal4;
    let transformed_norm = Vec3::new(transformed_normal.x, transformed_normal.y, transformed_normal.z);
    let transformed_norm = if transformed_norm.magnitude() > 0.001 {
        transformed_norm.normalize()
    } else {
        Vec3::new(0.0, 1.0, 0.0)
    };

    Vertex {
        position: vertex.position,
        normal: vertex.normal,
        tex_coords: vertex.tex_coords,
        color: vertex.color,
        transformed_position: transformed_pos,
        transformed_normal: transformed_norm,
    }
}
