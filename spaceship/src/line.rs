use crate::vertex::Vertex;
use crate::framebuffer::Framebuffer;

pub fn line(a: &Vertex, b: &Vertex, framebuffer: &mut Framebuffer) {
    let start = a.transformed_position;
    let end = b.transformed_position;

    let mut x0 = start.x as i32;
    let mut y0 = start.y as i32;
    let x1 = end.x as i32;
    let y1 = end.y as i32;

    let dx = (x1 - x0).abs();
    let dy = (y1 - y0).abs();

    let sx = if x0 < x1 { 1 } else { -1 };
    let sy = if y0 < y1 { 1 } else { -1 };

    let mut err = if dx > dy { dx / 2 } else { -dy / 2 };

    loop {
        if x0 >= 0 && x0 < framebuffer.width as i32 && y0 >= 0 && y0 < framebuffer.height as i32 {
            let z = if (end.x - start.x).abs() > 0.0001 {
                start.z + (end.z - start.z) * (x0 as f32 - start.x) / (end.x - start.x)
            } else {
                start.z
            };
            framebuffer.point(x0 as usize, y0 as usize, z);
        }

        if x0 == x1 && y0 == y1 { break; }

        let e2 = err;
        if e2 > -dx {
            err -= dy;
            x0 += sx;
        }
        if e2 < dy {
            err += dx;
            y0 += sy;
        }
    }
}
