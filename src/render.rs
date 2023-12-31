use std::mem::size_of;
use std::ops::{Add, Div, Mul, Sub};

use bytemuck::offset_of;
use gl33::{global_loader::*, GL_ARRAY_BUFFER, GL_DYNAMIC_DRAW, GL_FALSE, GL_FLOAT, GL_TRIANGLES};

use crate::font::FontAtlas;

pub const FPS: u32 = 60;
pub const DELTA_TIME_MS: u32 = 1000 / FPS;
pub const DELTA_TIME: f32 = 1000.0 / (FPS as f32);

const MAX_VERTICES: usize = 10 * 640 * 1000;

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct V2 {
    pub x: f32,
    pub y: f32,
}

impl From<(f32, f32)> for V2 {
    fn from(value: (f32, f32)) -> Self {
        V2 {
            x: value.0,
            y: value.1,
        }
    }
}

impl From<(i32, i32)> for V2 {
    fn from(value: (i32, i32)) -> Self {
        V2 {
            x: value.0 as f32,
            y: value.1 as f32,
        }
    }
}

impl Add<V2> for V2 {
    type Output = V2;

    fn add(self, rhs: V2) -> Self::Output {
        V2 {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl Sub<V2> for V2 {
    type Output = V2;

    fn sub(self, rhs: V2) -> Self::Output {
        V2 {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl Mul<f32> for V2 {
    type Output = V2;

    fn mul(self, rhs: f32) -> Self::Output {
        V2 {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

impl Mul<V2> for V2 {
    type Output = V2;

    fn mul(self, rhs: V2) -> Self::Output {
        V2 {
            x: self.x * rhs.x,
            y: self.y * rhs.y,
        }
    }
}

impl Div<f32> for V2 {
    type Output = V2;

    fn div(self, rhs: f32) -> Self::Output {
        V2 {
            x: self.x / rhs,
            y: self.y / rhs,
        }
    }
}

#[derive(Clone, Copy, Default)]
pub struct V4 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub a: f32,
}

impl V4 {
    pub fn rgb(r: f32, g: f32, b: f32) -> V4 {
        V4::rgba(r, g, b, 1.0)
    }

    pub fn rgba(r: f32, g: f32, b: f32, a: f32) -> V4 {
        V4 {
            x: r,
            y: g,
            z: b,
            a,
        }
    }
}

#[derive(Clone, Copy, Default)]
pub struct Vertex {
    pub pos: V2,
    pub color: V4,
    pub uv: V2,
}

impl Vertex {
    pub fn new(pos: V2, color: V4, uv: V2) -> Vertex {
        Vertex { pos, color, uv }
    }
}

pub struct Renderer {
    vao: u32,
    vbo: u32,
    pub vertices: Vec<Vertex>,
}

impl Renderer {
    pub fn new() -> Renderer {
        let mut renderer = Renderer {
            vao: 0,
            vbo: 0,
            vertices: Vec::with_capacity(MAX_VERTICES),
        };

        unsafe {
            glGenVertexArrays(1, &mut renderer.vao);
            glBindVertexArray(renderer.vao);

            glGenBuffers(1, &mut renderer.vbo);
            glBindBuffer(GL_ARRAY_BUFFER, renderer.vbo);
            glBufferData(
                GL_ARRAY_BUFFER,
                (size_of::<Vertex>() * MAX_VERTICES) as isize,
                renderer.vertices.as_ptr().cast(),
                GL_DYNAMIC_DRAW,
            );

            // position
            glEnableVertexAttribArray(0);
            glVertexAttribPointer(
                0, // location 0
                2, // 2 values (V2)
                GL_FLOAT,
                GL_FALSE.0 as u8,
                size_of::<Vertex>() as i32,
                offset_of!(Vertex, pos) as *const _,
            );

            // color
            glEnableVertexAttribArray(1);
            glVertexAttribPointer(
                1, // location 1
                4, // 4 values (V4)
                GL_FLOAT,
                GL_FALSE.0 as u8,
                size_of::<Vertex>() as i32,
                offset_of!(Vertex, color) as *const _,
            );

            // uv
            glEnableVertexAttribArray(2);
            glVertexAttribPointer(
                2, // location 2
                2, // 2 values (V2)
                GL_FLOAT,
                GL_FALSE.0 as u8,
                size_of::<Vertex>() as i32,
                offset_of!(Vertex, uv) as *const _,
            );
        }

        renderer
    }

    pub fn flush(&mut self) {
        unsafe {
            glBufferSubData(
                GL_ARRAY_BUFFER,
                0,
                (size_of::<Vertex>() * self.vertices.len()) as isize,
                self.vertices.as_mut_ptr().cast(),
            );
            glDrawArrays(GL_TRIANGLES, 0, self.vertices.len() as i32);
        }
        self.vertices.clear();
    }

    pub fn render_vertex(&mut self, v: Vertex) {
        self.vertices.push(v);
    }

    pub fn render_triangle(&mut self, v0: Vertex, v1: Vertex, v2: Vertex) {
        self.render_vertex(v0);
        self.render_vertex(v1);
        self.render_vertex(v2);
    }

    pub fn render_quad(&mut self, v0: Vertex, v1: Vertex, v2: Vertex, v3: Vertex) {
        self.render_triangle(v0, v1, v2);
        self.render_triangle(v1, v2, v3);
    }

    pub fn render_solid_rect(&mut self, p: V2, s: V2, c: V4) {
        let uvp = V2::default();
        let p1 = p + V2 { x: s.x, y: 0.0 };
        let p2 = p + V2 { x: 0.0, y: s.y };
        let p3 = p + s;

        self.render_quad(
            Vertex::new(p, c, uvp),
            Vertex::new(p1, c, uvp),
            Vertex::new(p2, c, uvp),
            Vertex::new(p3, c, uvp),
        )
    }

    pub fn render_image_rect(&mut self, p: V2, s: V2, uvp: V2, uvs: V2, c: V4) {
        let p1 = p + V2 { x: s.x, y: 0.0 };
        let p2 = p + V2 { x: 0.0, y: s.y };
        let p3 = p + s;
        let uv1 = uvp + V2 { x: uvs.x, y: 0.0 };
        let uv2 = uvp + V2 { x: 0.0, y: uvs.y };
        let uv3 = uvp + uvs;

        self.render_quad(
            Vertex::new(p, c, uvp),
            Vertex::new(p1, c, uv1),
            Vertex::new(p2, c, uv2),
            Vertex::new(p3, c, uv3),
        )
    }

    pub fn render_text(
        &mut self,
        atlas: &FontAtlas,
        text: &str,
        mut pos: V2,
        color: V4,
        scale: f32,
    ) -> f32 {
        let mut width = 0.0;

        for c in text.chars() {
            let glyph = atlas.glyph(c);

            let x = pos.x + (glyph.bl as f32 * scale);
            let y = -pos.y - (glyph.bt as f32 * scale);

            pos.x += glyph.ax * scale;
            pos.y += glyph.ay * scale;

            self.render_image_rect(
                V2 { x, y: -y },
                V2 {
                    x: glyph.bw as f32 * scale,
                    y: -glyph.bh as f32 * scale,
                },
                V2 {
                    x: glyph.tx,
                    y: 0.0,
                },
                V2 {
                    x: (glyph.bw as f32) / (atlas.atlas_width as f32),
                    y: (glyph.bh as f32) / (atlas.atlas_height as f32),
                },
                color,
            );

            width += glyph.ax;
        }

        width
    }
}
