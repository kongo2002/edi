use std::mem::size_of;

use beryllium::error::SdlError;
use beryllium::init::InitFlags;
use beryllium::{events, video, Sdl};
use bytemuck::offset_of;
use gl33::{
    global_loader::*, GL_ARRAY_BUFFER, GL_BLEND, GL_COLOR_BUFFER_BIT, GL_COMPILE_STATUS,
    GL_DYNAMIC_DRAW, GL_FALSE, GL_FLOAT, GL_FRAGMENT_SHADER, GL_LINK_STATUS,
    GL_ONE_MINUS_SRC_ALPHA, GL_SRC_ALPHA, GL_TRIANGLES, GL_VERTEX_SHADER,
};

use crate::gl::GL;

use self::errors::EdiError;
use self::font::FontAtlas;

mod errors;
mod font;
mod gl;

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

#[derive(Clone, Copy, Default)]
struct V2 {
    pub x: f32,
    pub y: f32,
}

#[derive(Clone, Copy, Default)]
struct V4 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub a: f32,
}

#[derive(Clone, Copy, Default)]
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

const MAX_VERTICES: usize = 10 * 640 * 1000;

struct Renderer {
    pub vertices: Vec<Vertex0>,
}

impl Renderer {
    fn new() -> Renderer {
        Renderer {
            vertices: Vec::with_capacity(MAX_VERTICES),
        }
    }

    fn flush(&mut self) {
        unsafe {
            glBufferSubData(
                GL_ARRAY_BUFFER,
                0,
                (size_of::<Vertex0>() * self.vertices.len()) as isize,
                self.vertices.as_mut_ptr().cast(),
            );
            glDrawArrays(GL_TRIANGLES, 0, self.vertices.len() as i32);
        }
        self.vertices.clear();
    }
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

    let program;
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
            (size_of::<Vertex0>() * MAX_VERTICES) as isize,
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
            size_of::<Vertex0>() as i32,
            offset_of!(Vertex0, pos) as *const _,
        );

        // color
        glEnableVertexAttribArray(1);
        glVertexAttribPointer(
            1, // location 1
            4, // 4 values (V4)
            GL_FLOAT,
            GL_FALSE.0 as u8,
            size_of::<Vertex0>() as i32,
            offset_of!(Vertex0, color) as *const _,
        );

        // uv
        glEnableVertexAttribArray(2);
        glVertexAttribPointer(
            2, // location 2
            2, // 2 values (V2)
            GL_FLOAT,
            GL_FALSE.0 as u8,
            size_of::<Vertex0>() as i32,
            offset_of!(Vertex0, uv) as *const _,
        );

        let vertex_shader = GL::create_shader(GL_VERTEX_SHADER, &vert_glsl)?;
        let fragment_shader = GL::create_shader(GL_FRAGMENT_SHADER, &frag_glsl)?;

        program = GL::create_program(&[vertex_shader, fragment_shader])?;
        program.use_program();

        glEnable(GL_BLEND);
        glBlendFunc(GL_SRC_ALPHA, GL_ONE_MINUS_SRC_ALPHA);
    }

    let font_atlas = FontAtlas::new("iosevka.ttf")?;

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

            let loc = glGetUniformLocation(program.id, b"resolution\0".as_ptr());
            glUniform2f(loc, win_width as f32, win_height as f32);
        }

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
            V2 {
                x: -500.0,
                y: 300.0,
            },
            color,
        );

        renderer.flush();
        win.swap_window();
    }

    Ok(())
}

fn render_vertex(renderer: &mut Renderer, v0: Vertex0) {
    renderer.vertices.push(v0);
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

fn render_text(renderer: &mut Renderer, atlas: &FontAtlas, text: &str, mut pos: V2, color: V4) {
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

        render_image_rect(
            renderer,
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

fn main() {
    match run() {
        Ok(()) => {}
        Err(err) => {
            eprintln!("{err}");
            std::process::exit(1);
        }
    }
}
