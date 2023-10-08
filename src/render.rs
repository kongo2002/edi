use std::mem::size_of;

use bytemuck::offset_of;
use gl33::{global_loader::*, GL_ARRAY_BUFFER, GL_DYNAMIC_DRAW, GL_FALSE, GL_FLOAT, GL_TRIANGLES};

use crate::font::FontAtlas;

const MAX_VERTICES: usize = 10 * 640 * 1000;

#[derive(Clone, Copy, Default)]
pub struct V2 {
    pub x: f32,
    pub y: f32,
}

#[derive(Clone, Copy, Default)]
pub struct V4 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub a: f32,
}

#[derive(Clone, Copy, Default)]
pub struct Vertex {
    pub pos: V2,
    pub color: V4,
    pub uv: V2,
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

    pub fn render_vertex(&mut self, v0: Vertex) {
        self.vertices.push(v0);
    }

    pub fn render_triangle(&mut self, v0: Vertex, v1: Vertex, v2: Vertex) {
        self.render_vertex(v0);
        self.render_vertex(v1);
        self.render_vertex(v2);
    }

    pub fn render_quad(
        &mut self,
        p0: V2,
        p1: V2,
        p2: V2,
        p3: V2,
        c0: V4,
        c1: V4,
        c2: V4,
        c3: V4,
        uv0: V2,
        uv1: V2,
        uv2: V2,
        uv3: V2,
    ) {
        self.render_triangle(
            Vertex {
                pos: p0,
                color: c0,
                uv: uv0,
            },
            Vertex {
                pos: p1.clone(),
                color: c1.clone(),
                uv: uv1.clone(),
            },
            Vertex {
                pos: p2.clone(),
                color: c2.clone(),
                uv: uv2.clone(),
            },
        );

        self.render_triangle(
            Vertex {
                pos: p1,
                color: c1,
                uv: uv1,
            },
            Vertex {
                pos: p2,
                color: c2,
                uv: uv2,
            },
            Vertex {
                pos: p3,
                color: c3,
                uv: uv3,
            },
        );
    }

    pub fn render_image_rect(&mut self, p: V2, s: V2, uvp: V2, uvs: V2, c: V4) {
        let p1 = V2 {
            x: p.x + s.x,
            y: p.y,
        };
        let p2 = V2 {
            x: p.x,
            y: p.y + s.y,
        };
        let p3 = V2 {
            x: p.x + s.x,
            y: p.y + s.y,
        };
        let uv1 = V2 {
            x: uvp.x + uvs.x,
            y: uvp.y,
        };
        let uv2 = V2 {
            x: uvp.x,
            y: uvp.y + uvs.y,
        };
        let uv3 = V2 {
            x: uvp.x + uvs.x,
            y: uvp.y + uvs.y,
        };
        self.render_quad(
            p,
            p1,
            p2,
            p3,
            c.clone(),
            c.clone(),
            c.clone(),
            c,
            uvp,
            uv1,
            uv2,
            uv3,
        )
    }

    pub fn render_text(&mut self, atlas: &FontAtlas, text: &str, mut pos: V2, color: V4) {
        for c in text.chars() {
            let idx = if c as usize >= 128 {
                '?' as usize
            } else {
                c as usize
            };
            let glyph = &atlas.glyphs[idx];

            let x2 = pos.x + (glyph.bl as f32);
            let y2 = -pos.y - (glyph.bt as f32);

            pos.x += glyph.ax;
            pos.y += glyph.ay;

            self.render_image_rect(
                V2 { x: x2, y: -y2 },
                V2 {
                    x: glyph.bw as f32,
                    y: -glyph.bh as f32,
                },
                V2 {
                    x: glyph.tx,
                    y: 0.0,
                },
                V2 {
                    x: (glyph.bw as f32) / (atlas.atlas_width as f32),
                    y: (glyph.bh as f32) / (atlas.atlas_height as f32),
                },
                color.clone(),
            );
        }
    }
}
