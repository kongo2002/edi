use std::mem::size_of;

use beryllium::error::SdlError;
use beryllium::init::InitFlags;
use beryllium::{events, video, Sdl};
use bytemuck::offset_of;
use gl33::{
    global_loader::*, GL_ARRAY_BUFFER, GL_BLEND, GL_COLOR_BUFFER_BIT, GL_DYNAMIC_DRAW, GL_FALSE,
    GL_FLOAT, GL_FRAGMENT_SHADER, GL_ONE_MINUS_SRC_ALPHA, GL_SRC_ALPHA, GL_VERTEX_SHADER,
};

use crate::gl::GL;

use self::errors::EdiError;
use self::font::FontAtlas;
use self::render::{Vertex, V2, V4, MAX_VERTICES};

mod errors;
mod font;
mod gl;
mod render;

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
    let mut renderer = render::Renderer::new();
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

        let vertex_shader = GL::create_shader(GL_VERTEX_SHADER, &vert_glsl)?;
        let fragment_shader = GL::create_shader(GL_FRAGMENT_SHADER, &frag_glsl)?;

        program = GL::create_program(&[vertex_shader, fragment_shader])?;
        program.use_program();

        glEnable(GL_BLEND);
        glBlendFunc(GL_SRC_ALPHA, GL_ONE_MINUS_SRC_ALPHA);
    }

    let resolution_uniform = program.get_location("resolution")?;
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

            glUniform2f(resolution_uniform, win_width as f32, win_height as f32);
        }

        let color = V4 {
            x: 1.0,
            y: 1.0,
            z: 0.1,
            a: 1.0,
        };

        renderer.render_text(
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

fn main() {
    match run() {
        Ok(()) => {}
        Err(err) => {
            eprintln!("{err}");
            std::process::exit(1);
        }
    }
}
