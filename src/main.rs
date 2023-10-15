use beryllium::error::SdlError;
use beryllium::init::InitFlags;
use beryllium::{events, video, Sdl};
use gl33::{global_loader::*, GL_BLEND, GL_COLOR_BUFFER_BIT, GL_ONE_MINUS_SRC_ALPHA, GL_SRC_ALPHA};

use crate::gl::GL;

use self::camera::Camera;
use self::errors::EdiError;
use self::font::{FontAtlas, FONT_PIXEL_HEIGHT};
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

    let vert_glsl = std::fs::read_to_string("shaders/vert.glsl")?;
    let text_frag_glsl = std::fs::read_to_string("shaders/text.glsl")?;
    let color_frag_glsl = std::fs::read_to_string("shaders/color.glsl")?;

    let win = sdl.create_gl_window(win_args).map_err(sdl_error)?;
    win.set_swap_interval(video::GlSwapInterval::Vsync)
        .map_err(sdl_error)?;

    unsafe {
        load_global_gl(&|f_name| win.get_proc_address(f_name));

        glEnable(GL_BLEND);
        glBlendFunc(GL_SRC_ALPHA, GL_ONE_MINUS_SRC_ALPHA);
    }

    let mut gl = GL::new();
    let mut renderer = render::Renderer::new();

    let text_shader = gl.create_program(&vert_glsl, &text_frag_glsl)?;
    let color_shader = gl.create_program(&vert_glsl, &color_frag_glsl)?;

    let font_atlas = FontAtlas::new("iosevka.ttf")?;

    let camera = Camera {
        pos: V2::default(),
        velocity: V2::default(),
        scale: 1.0,
        scale_velocity: 1.0,
    };

    let mut text = String::with_capacity(256);

    'main_loop: loop {
        while let Some((event, _ts)) = sdl.poll_events() {
            match event {
                events::Event::Quit => break 'main_loop,
                events::Event::TextInput {
                    win_id: _,
                    text: input,
                } => text.push_str(&input),
                events::Event::Key {
                    win_id,
                    pressed: true,
                    repeat,
                    scancode,
                    keycode,
                    modifiers,
                } => match keycode {
                    fermium::keycode::SDLK_BACKSPACE => {
                        text.pop();
                    }
                    _ => (),
                },

                _ => (),
            }
        }

        let (win_width, win_height) = win.get_window_size().into();
        let resolution = (win_width, win_height).into();

        unsafe {
            // TODO: necessary all the time?
            glViewport(0, 0, win_width, win_width);
            glClearColor(0.2, 0.3, 0.3, 1.0);
            glClear(GL_COLOR_BUFFER_BIT);
        }

        // render text
        {
            text_shader.activate(&resolution, &camera);

            let text_color = V4 {
                x: 1.0,
                y: 1.0,
                z: 0.1,
                a: 1.0,
            };

            renderer.render_text(&font_atlas, &text, (-500.0, 300.0).into(), text_color);
            renderer.flush();
        }

        // render cursor
        {
            color_shader.activate(&resolution, &camera);

            let cursor_size = ((FONT_PIXEL_HEIGHT as f32) / 6.0, FONT_PIXEL_HEIGHT as f32);
            let cursor_color = V4 {
                x: 1.0,
                y: 0.3,
                z: 0.3,
                a: 1.0,
            };

            renderer.render_solid_rect((-400.0, 300.0).into(), cursor_size.into(), cursor_color);
            renderer.flush();
        }

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
