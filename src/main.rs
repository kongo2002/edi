use std::mem::{size_of, size_of_val};

use beryllium::error::SdlError;
use beryllium::init::InitFlags;
use beryllium::{events, video, Sdl};
use freetype::face::LoadFlag;
use gl33::{
    global_loader::*, GL_ARRAY_BUFFER, GL_BLEND, GL_CLAMP_TO_EDGE, GL_COLOR_BUFFER_BIT,
    GL_COMPILE_STATUS, GL_DYNAMIC_DRAW, GL_FALSE, GL_FLOAT, GL_FRAGMENT_SHADER, GL_LINEAR,
    GL_LINK_STATUS, GL_ONE_MINUS_SRC_ALPHA, GL_RED, GL_SRC_ALPHA, GL_TEXTURE0,
    GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER, GL_TEXTURE_MIN_FILTER, GL_TEXTURE_WRAP_S,
    GL_TEXTURE_WRAP_T, GL_TRIANGLES, GL_UNPACK_ALIGNMENT, GL_UNSIGNED_BYTE, GL_VERTEX_SHADER,
};

use self::errors::EdiError;

mod errors;

fn init_sdl() -> Result<Sdl, EdiError> {
    let sdl = Sdl::init(InitFlags::VIDEO | InitFlags::EVENTS);
    sdl.set_gl_context_major_version(3).map_err(sdl_error)?;
    sdl.set_gl_context_minor_version(3).map_err(sdl_error)?;

    #[cfg(target_os = "macos")]
    {
        sdl.set_gl_context_flags(video::GlContextFlags::FORWARD_COMPATIBLE)
            .map_err(sdl_error)?;
    }

    Ok(sdl)
}

#[derive(Clone, Copy)]
struct V2 {
    pub x: f32,
    pub y: f32,
}

#[derive(Clone, Copy)]
struct V4 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub a: f32,
}

#[derive(Clone, Copy)]
struct Vertex0 {
    pub pos: V2,
    pub color: V4,
    pub uv: V2,
}

impl Vertex0 {
    fn new() -> Vertex0 {
        Vertex0 {
            pos: V2 { x: 0.0, y: 0.0 },
            color: V4 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
                a: 0.0,
            },
            uv: V2 { x: 0.0, y: 0.0 },
        }
    }
}

const MAX_VERTICES: usize = 1000; //10 * 640 * 1000;

struct Renderer {
    pub vertices: [Vertex0; MAX_VERTICES],
    pub num_vertices: usize,
}

impl Renderer {
    fn new() -> Renderer {
        Renderer {
            vertices: [Vertex0::new(); MAX_VERTICES],
            num_vertices: 0,
        }
    }

    fn flush(&mut self) {
        unsafe {
            glBufferSubData(
                GL_ARRAY_BUFFER,
                0,
                (size_of::<Vertex0>() * self.num_vertices) as isize,
                self.vertices.as_ptr().cast(),
            );
            glDrawArrays(GL_TRIANGLES, 0, self.num_vertices as i32);
        }
        self.num_vertices = 0;
    }
}

#[derive(Clone, Copy)]
struct GlyphInfo {
    ax: f32, // advance X
    ay: f32, // advance Y
    bw: f32, // bitmap width
    bh: f32, // bitmap height
    bl: f32, // bitmap left
    bt: f32, // bitmap top
    tx: f32, // x offset of glyph in texture coordinates
}

impl GlyphInfo {
    fn new() -> GlyphInfo {
        GlyphInfo {
            ax: 0.0,
            ay: 0.0,
            bw: 0.0,
            bh: 0.0,
            bl: 0.0,
            bt: 0.0,
            tx: 0.0,
        }
    }
}

struct FontAtlas {
    pub texture: u32,
    pub atlas_height: i32,
    pub atlas_width: i32,
    pub glyphs: [GlyphInfo; 128],
}

fn init_font(font: &str) -> Result<FontAtlas, EdiError> {
    let library = freetype::Library::init()?;
    let face = library.new_face(font, 0)?;

    face.set_pixel_sizes(0, 64)?;

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

        let mut x = 0.0;

        for i in 32..128usize {
            face.load_char(i, LoadFlag::RENDER)?;
            face.glyph().render_glyph(freetype::RenderMode::Normal)?;

            glyphs[i].ax = (face.glyph().advance().x >> 6) as f32;
            glyphs[i].ay = (face.glyph().advance().y >> 6) as f32;
            glyphs[i].bw = face.glyph().bitmap().width() as f32;
            glyphs[i].bh = face.glyph().bitmap().rows() as f32;
            glyphs[i].bl = face.glyph().bitmap_left() as f32;
            glyphs[i].bt = face.glyph().bitmap_top() as f32;
            glyphs[i].tx = x / (atlas_width as f32);

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

fn sdl_error(err: SdlError) -> EdiError {
    EdiError::SdlError(format!("{:?}", err))
}

fn run() -> Result<(), EdiError> {
    let sdl = init_sdl()?;

    let win_args = video::CreateWinArgs {
        title: "edi",
        width: 800,
        height: 600,
        allow_high_dpi: true,
        borderless: false,
        resizable: true,
    };

    let mut shader_program;
    let mut renderer = Renderer::new();
    let vert_glsl = std::fs::read_to_string("vert.glsl")?;
    let frag_glsl = std::fs::read_to_string("frag.glsl")?;

    let win = sdl.create_gl_window(win_args).map_err(sdl_error)?;
    win.set_swap_interval(video::GlSwapInterval::Vsync)
        .map_err(sdl_error)?;

    unsafe {
        load_global_gl(&|f_name| win.get_proc_address(f_name));

        let mut vao = 0;
        glGenVertexArrays(1, &mut vao);
        glBindVertexArray(vao);

        let mut vbo = 0;
        glGenBuffers(1, &mut vbo);
        glBindBuffer(GL_ARRAY_BUFFER, vbo);
        glBufferData(
            GL_ARRAY_BUFFER,
            size_of_val(&renderer.vertices) as isize,
            renderer.vertices.as_ptr().cast(),
            GL_DYNAMIC_DRAW,
        );

        // position
        glEnableVertexAttribArray(0);
        glVertexAttribPointer(
            0,
            2,
            GL_FLOAT,
            GL_FALSE.0 as u8,
            size_of::<Vertex0>() as i32,
            16 as *const _,
        );

        // color
        glEnableVertexAttribArray(1);
        glVertexAttribPointer(
            1,
            4,
            GL_FLOAT,
            GL_FALSE.0 as u8,
            size_of::<Vertex0>() as i32,
            0 as *const _,
        );

        // uv
        glEnableVertexAttribArray(2);
        glVertexAttribPointer(
            2,
            2,
            GL_FLOAT,
            GL_FALSE.0 as u8,
            size_of::<Vertex0>() as i32,
            24 as *const _,
        );

        let vertex_shader = glCreateShader(GL_VERTEX_SHADER);
        assert_ne!(vertex_shader, 0);
        glShaderSource(
            vertex_shader,
            1,
            &(vert_glsl.as_bytes().as_ptr().cast()),
            &(vert_glsl.len().try_into().unwrap()),
        );
        glCompileShader(vertex_shader);
        let mut success = 0;
        glGetShaderiv(vertex_shader, GL_COMPILE_STATUS, &mut success);
        if success == 0 {
            let mut v: Vec<u8> = Vec::with_capacity(1024);
            let mut log_len = 0_i32;
            glGetShaderInfoLog(vertex_shader, 1024, &mut log_len, v.as_mut_ptr().cast());
            v.set_len(log_len.try_into().unwrap());
            panic!("Vertex Compile Error: {}", String::from_utf8_lossy(&v));
        }

        let fragment_shader = glCreateShader(GL_FRAGMENT_SHADER);
        assert_ne!(fragment_shader, 0);
        glShaderSource(
            fragment_shader,
            1,
            &(frag_glsl.as_bytes().as_ptr().cast()),
            &(frag_glsl.len().try_into().unwrap()),
        );
        glCompileShader(fragment_shader);
        let mut success = 0;
        glGetShaderiv(fragment_shader, GL_COMPILE_STATUS, &mut success);
        if success == 0 {
            let mut v: Vec<u8> = Vec::with_capacity(1024);
            let mut log_len = 0_i32;
            glGetShaderInfoLog(fragment_shader, 1024, &mut log_len, v.as_mut_ptr().cast());
            v.set_len(log_len.try_into().unwrap());
            panic!("Fragment Compile Error: {}", String::from_utf8_lossy(&v));
        }

        shader_program = glCreateProgram();
        assert_ne!(shader_program, 0);
        glAttachShader(shader_program, vertex_shader);
        glAttachShader(shader_program, fragment_shader);
        glLinkProgram(shader_program);
        let mut success = 0;
        glGetProgramiv(shader_program, GL_LINK_STATUS, &mut success);
        if success == 0 {
            let mut v: Vec<u8> = Vec::with_capacity(1024);
            let mut log_len = 0_i32;
            glGetProgramInfoLog(shader_program, 1024, &mut log_len, v.as_mut_ptr().cast());
            v.set_len(log_len.try_into().unwrap());
            panic!("Program Link Error: {}", String::from_utf8_lossy(&v));
        }
        glDeleteShader(vertex_shader);
        glDeleteShader(fragment_shader);

        glUseProgram(shader_program);

        glEnable(GL_BLEND);
        glBlendFunc(GL_SRC_ALPHA, GL_ONE_MINUS_SRC_ALPHA);
    }

    let font_atlas = init_font("iosevka.ttf")?;

    'main_loop: loop {
        while let Some(event) = sdl.poll_events() {
            match event {
                (events::Event::Quit, _) => break 'main_loop,
                _ => (),
            }
        }

        let (win_width, win_height) = win.get_window_size();

        unsafe {
            glViewport(0, 0, win_width, win_height);
            glClearColor(0.2, 0.3, 0.3, 1.0);
            glClear(GL_COLOR_BUFFER_BIT);

            let loc = glGetUniformLocation(shader_program, "resolution\x00".as_bytes().as_ptr());
            glUniform2f(loc, win_width as f32, win_height as f32);
        }

        let mut pos = V2 {
            x: -500.0,
            y: 300.0,
        };
        let color = V4 {
            x: 1.0,
            y: 1.0,
            z: 0.1,
            a: 1.0,
        };

        render_text(
            &mut renderer,
            &font_atlas,
            "this is a test",
            &mut pos,
            color,
        );

        renderer.flush();
        win.swap_window();
    }

    Ok(())
}

fn render_vertex(renderer: &mut Renderer, v0: Vertex0) {
    if renderer.num_vertices == MAX_VERTICES - 1 {
        return;
    }
    renderer.vertices[renderer.num_vertices].pos = v0.pos;
    renderer.vertices[renderer.num_vertices].color = v0.color;
    renderer.vertices[renderer.num_vertices].uv = v0.uv;

    renderer.num_vertices += 1;
}

fn render_triangle(renderer: &mut Renderer, v0: Vertex0, v1: Vertex0, v2: Vertex0) {
    render_vertex(renderer, v0);
    render_vertex(renderer, v1);
    render_vertex(renderer, v2);
}

fn render_quad(
    renderer: &mut Renderer,
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
    render_triangle(
        renderer,
        Vertex0 {
            pos: p0,
            color: c0,
            uv: uv0,
        },
        Vertex0 {
            pos: p1.clone(),
            color: c1.clone(),
            uv: uv1.clone(),
        },
        Vertex0 {
            pos: p2.clone(),
            color: c2.clone(),
            uv: uv2.clone(),
        },
    );

    render_triangle(
        renderer,
        Vertex0 {
            pos: p1,
            color: c1,
            uv: uv1,
        },
        Vertex0 {
            pos: p2,
            color: c2,
            uv: uv2,
        },
        Vertex0 {
            pos: p3,
            color: c3,
            uv: uv3,
        },
    );
}

fn render_image_rect(renderer: &mut Renderer, p: V2, s: V2, uvp: V2, uvs: V2, c: V4) {
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
    render_quad(
        renderer,
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

fn render_text(renderer: &mut Renderer, atlas: &FontAtlas, text: &str, pos: &mut V2, color: V4) {
    for c in text.chars() {
        let idx = if c as usize >= 128 {
            '?' as usize
        } else {
            c as usize
        };
        let glyph = &atlas.glyphs[idx];

        let x2 = pos.x + glyph.bl;
        let y2 = -pos.y - glyph.bt;

        pos.x += glyph.ax;
        pos.y += glyph.ay;

        render_image_rect(
            renderer,
            V2 { x: x2, y: -y2 },
            V2 {
                x: glyph.bw,
                y: -glyph.bh,
            },
            V2 {
                x: glyph.tx,
                y: 0.0,
            },
            V2 {
                x: glyph.bw / (atlas.atlas_width as f32),
                y: glyph.bh / (atlas.atlas_height as f32),
            },
            color.clone(),
        );
    }
}

fn main() {
    match run() {
        Ok(()) => {}
        Err(err) => {
            eprintln!("{err}");
            std::process::exit(1);
        }
    }
}
