use freetype::face::LoadFlag;
use gl33::{
    global_loader::*, GL_LINEAR, GL_RED, GL_TEXTURE0, GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER,
    GL_TEXTURE_MIN_FILTER, GL_TEXTURE_WRAP_S, GL_TEXTURE_WRAP_T, GL_UNPACK_ALIGNMENT,
    GL_UNSIGNED_BYTE, GL_CLAMP_TO_EDGE,
};

use crate::errors::EdiError;

#[derive(Clone, Copy)]
pub struct GlyphInfo {
    pub ax: f32, // advance X
    pub ay: f32, // advance Y
    pub bw: i32, // bitmap width
    pub bh: i32, // bitmap height
    pub bl: i32, // bitmap left
    pub bt: i32, // bitmap top
    pub tx: f32, // x offset of glyph in texture coordinates
}

impl GlyphInfo {
    fn new() -> GlyphInfo {
        GlyphInfo {
            ax: 0.0,
            ay: 0.0,
            bw: 0,
            bh: 0,
            bl: 0,
            bt: 0,
            tx: 0.0,
        }
    }
}

pub struct FontAtlas {
    pub texture: u32,
    pub atlas_height: i32,
    pub atlas_width: i32,
    pub glyphs: [GlyphInfo; 128],
}

const FONT_PIXEL_HEIGHT: u32 = 256;

impl FontAtlas {
    pub fn new(font: &str) -> Result<FontAtlas, EdiError> {
        let library = freetype::Library::init()?;
        let face = library.new_face(font, 0)?;

        face.set_pixel_sizes(0, FONT_PIXEL_HEIGHT)?;

        let mut atlas_width = 0i32;
        let mut atlas_height = 0i32;

        for i in 32..128usize {
            face.load_char(i, LoadFlag::RENDER)?;

            atlas_width += face.glyph().raw().bitmap.width;
            atlas_height = atlas_height.max(face.glyph().raw().bitmap.rows);
        }

        let mut glyphs = [GlyphInfo::new(); 128];
        let mut glyph_texture = 0u32;

        unsafe {
            glActiveTexture(GL_TEXTURE0);
            glGenTextures(1, &mut glyph_texture);
            glBindTexture(GL_TEXTURE_2D, glyph_texture);

            glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER, GL_LINEAR.0 as i32);
            glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER, GL_LINEAR.0 as i32);
            glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_S, GL_CLAMP_TO_EDGE.0 as i32);
            glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_T, GL_CLAMP_TO_EDGE.0 as i32);

            glPixelStorei(GL_UNPACK_ALIGNMENT, 1);

            glTexImage2D(
                GL_TEXTURE_2D,
                0,
                GL_RED.0 as i32,
                atlas_width,
                atlas_height,
                0,
                GL_RED,
                GL_UNSIGNED_BYTE,
                0 as *const _,
            );

            let mut x = 0;

            for i in 32..128usize {
                face.load_char(i, LoadFlag::RENDER)?;

                // TODO: freetype-rs does not support SDF yet :|
                face.glyph().render_glyph(freetype::RenderMode::Normal)?;

                glyphs[i].ax = (face.glyph().advance().x >> 6) as f32;
                glyphs[i].ay = (face.glyph().advance().y >> 6) as f32;
                glyphs[i].bw = face.glyph().bitmap().width();
                glyphs[i].bh = face.glyph().bitmap().rows();
                glyphs[i].bl = face.glyph().bitmap_left();
                glyphs[i].bt = face.glyph().bitmap_top();
                glyphs[i].tx = (x as f32) / (atlas_width as f32);

                glPixelStorei(GL_UNPACK_ALIGNMENT, 1);
                glTexSubImage2D(
                    GL_TEXTURE_2D,
                    0,
                    x as i32,
                    0,
                    glyphs[i].bw as i32,
                    glyphs[i].bh as i32,
                    GL_RED,
                    GL_UNSIGNED_BYTE,
                    face.glyph().bitmap().raw().buffer as *const _,
                );

                x += glyphs[i].bw;
            }
        }

        Ok(FontAtlas {
            texture: glyph_texture,
            atlas_height,
            atlas_width,
            glyphs,
        })
    }
}
