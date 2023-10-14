use beryllium::error::SdlError;
use beryllium::init::InitFlags;
use beryllium::{events, video, Sdl};
use gl33::{
    global_loader::*, GL_BLEND, GL_COLOR_BUFFER_BIT, GL_FRAGMENT_SHADER, GL_ONE_MINUS_SRC_ALPHA,
    GL_SRC_ALPHA, GL_VERTEX_SHADER,
};

use crate::gl::GL;

use self::camera::Camera;
use self::errors::EdiError;
use self::font::FontAtlas;
use self::render::{V2, V4};

mod camera;
mod errors;
mod font;
mod gl;
mod render;

fn init_sdl() -> Result<Sdl, EdiError> {
    let sdl = Sdl::init(InitFlags::VIDEO | InitFlags::EVENTS);
    sdl.set_gl_context_major_version(3).map_err(sdl_error)?;
    sdl.set_gl_context_minor_version(3).map_err(sdl_error)?;
    sdl.set_gl_profile(video::GlProfile::Core)
        .map_err(sdl_error)?;

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

    let vert_glsl = std::fs::read_to_string("vert.glsl")?;
    let frag_glsl = std::fs::read_to_string("frag.glsl")?;

    let win = sdl.create_gl_window(win_args).map_err(sdl_error)?;
    win.set_swap_interval(video::GlSwapInterval::Vsync)
        .map_err(sdl_error)?;

    unsafe {
        load_global_gl(&|f_name| win.get_proc_address(f_name));

        glEnable(GL_BLEND);
        glBlendFunc(GL_SRC_ALPHA, GL_ONE_MINUS_SRC_ALPHA);
    }

    let mut renderer = render::Renderer::new();

    let vertex_shader = GL::create_shader(GL_VERTEX_SHADER, &vert_glsl)?;
    let fragment_shader = GL::create_shader(GL_FRAGMENT_SHADER, &frag_glsl)?;

    let program = GL::create_program(&[vertex_shader, fragment_shader])?;
    program.use_program();

    let resolution_uniform = program.get_location("resolution")?;
    let camera_pos = program.get_location("camera_pos")?;
    let camera_scale = program.get_location("camera_scale")?;
    let font_atlas = FontAtlas::new("iosevka.ttf")?;

    let camera = Camera {
        pos: V2::default(),
        velocity: V2::default(),
        scale: 1.0,
        scale_velocity: 1.0,
    };

    'main_loop: loop {
        while let Some(event) = sdl.poll_events() {
            match event {
                (events::Event::Quit, _) => break 'main_loop,
                _ => (),
            }
        }

        let (win_width, win_height) = win.get_window_size();

        unsafe {
            // TODO: necessary all the time?
            glViewport(0, 0, win_width, win_height);
            glClearColor(0.2, 0.3, 0.3, 1.0);
            glClear(GL_COLOR_BUFFER_BIT);

            glUniform2f(resolution_uniform, win_width as f32, win_height as f32);

            glUniform1f(camera_scale, camera.scale);
            glUniform2f(camera_pos, camera.pos.x, camera.pos.y);
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
