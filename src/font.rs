use crossfont::{FontDesc, GlyphKey, Rasterize, Size, Slant, Weight};
use gl33::{
    global_loader::*, GL_CLAMP_TO_EDGE, GL_LINEAR, GL_RGB, GL_RGBA, GL_TEXTURE0, GL_TEXTURE_2D,
    GL_TEXTURE_MAG_FILTER, GL_TEXTURE_MIN_FILTER, GL_TEXTURE_WRAP_S, GL_TEXTURE_WRAP_T,
    GL_UNPACK_ALIGNMENT, GL_UNSIGNED_BYTE,
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

pub const FONT_SIZE: f32 = 256.0;

impl FontAtlas {
    pub fn new(font_name: &str) -> Result<FontAtlas, EdiError> {
        let device_pixel_ratio = 1.0;
        let mut rasterizer: crossfont::Rasterizer = crossfont::Rasterize::new(device_pixel_ratio)?;
        let desc = FontDesc::new(
            font_name,
            crossfont::Style::Description {
                slant: Slant::Normal,
                weight: Weight::Normal,
            },
        );
        let size = Size::new(256.0);
        let font_key = rasterizer.load_font(&desc, size)?;

        let mut atlas_width = 0i32;
        let mut atlas_height = 0i32;

        for i in 32..128u32 {
            let character = unsafe { char::from_u32_unchecked(i) };
            let glyph = rasterizer.get_glyph(GlyphKey {
                font_key,
                character,
                size,
            })?;

            atlas_width += glyph.width;
            atlas_height = atlas_height.max(glyph.height);
        }

        let mut glyphs = [GlyphInfo::new(); 128];
        let mut glyph_texture = 0u32;

        unsafe {
            glActiveTexture(GL_TEXTURE0);
            glPixelStorei(GL_UNPACK_ALIGNMENT, 1);
            glGenTextures(1, &mut glyph_texture);
            glBindTexture(GL_TEXTURE_2D, glyph_texture);

            glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER, GL_LINEAR.0 as i32);
            glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER, GL_LINEAR.0 as i32);
            glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_S, GL_CLAMP_TO_EDGE.0 as i32);
            glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_T, GL_CLAMP_TO_EDGE.0 as i32);

            glTexImage2D(
                GL_TEXTURE_2D,
                0,
                GL_RGBA.0 as i32,
                atlas_width,
                atlas_height,
                0,
                GL_RGBA,
                GL_UNSIGNED_BYTE,
                0 as *const _,
            );

            let mut x = 0;

            for i in 32..128usize {
                let character = char::from_u32_unchecked(i as u32);
                let glyph = rasterizer.get_glyph(GlyphKey {
                    font_key,
                    character,
                    size,
                })?;

                glyphs[i].ax = glyph.advance.0 as f32;
                glyphs[i].ay = glyph.advance.1 as f32;
                glyphs[i].bw = glyph.width;
                glyphs[i].bh = glyph.height;
                glyphs[i].bl = glyph.left;
                glyphs[i].bt = glyph.top;
                glyphs[i].tx = (x as f32) / (atlas_width as f32);

                let (format, buffer) = match &glyph.buffer {
                    crossfont::BitmapBuffer::Rgb(buffer) => (GL_RGB, buffer),
                    crossfont::BitmapBuffer::Rgba(buffer) => (GL_RGBA, buffer),
                };

                glTexSubImage2D(
                    GL_TEXTURE_2D,
                    0,
                    x,
                    0,
                    glyph.width,
                    glyph.height,
                    format,
                    GL_UNSIGNED_BYTE,
                    buffer.as_ptr() as *const _,
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

    pub fn glyph(&self, chr: char) -> &GlyphInfo {
        let idx = if chr as usize >= 128 {
            '?' as usize
        } else {
            chr as usize
        };
        &self.glyphs[idx]
    }
}
