use beryllium::error::SdlError;
use beryllium::init::InitFlags;
use beryllium::{events, video, Sdl};
use fermium::timer::SDL_Delay;
use gl33::{
    global_loader::*, GL_BLEND, GL_COLOR_BUFFER_BIT, GL_MULTISAMPLE, GL_ONE_MINUS_SRC_ALPHA,
    GL_SRC_ALPHA,
};

use crate::gl::GL;

use self::camera::Camera;
use self::cursor::{Cursor, CURSOR_OFFSET};
use self::editor::{Editor, Mode};
use self::errors::EdiError;
use self::font::{FontAtlas, FONT_PIXEL_HEIGHT};
use self::render::{DELTA_TIME, DELTA_TIME_MS, V2, V4};

mod camera;
mod cooldown;
mod cursor;
mod editor;
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
    let mut editor = if std::env::args().len() > 1 {
        Editor::from_file(&std::env::args().collect::<Vec<_>>()[1])
    } else {
        Ok(Editor::new())
    }?;

    let sdl = init_sdl()?;

    let win_args = video::CreateWinArgs {
        title: "edi",
        width: 800,
        height: 600,
        allow_high_dpi: true,
        borderless: false,
        resizable: true,
    };

    let camera_vert_glsl = std::fs::read_to_string("shaders/camera_vert.glsl")?;
    let ui_vert_glsl = std::fs::read_to_string("shaders/ui_vert.glsl")?;
    let text_frag_glsl = std::fs::read_to_string("shaders/text.glsl")?;
    let color_frag_glsl = std::fs::read_to_string("shaders/color.glsl")?;

    let win = sdl.create_gl_window(win_args).map_err(sdl_error)?;

    win.set_swap_interval(video::GlSwapInterval::Vsync)
        .map_err(sdl_error)?;

    unsafe {
        load_global_gl(&|f_name| win.get_proc_address(f_name));

        glEnable(GL_BLEND);
        glEnable(GL_MULTISAMPLE);
        glBlendFunc(GL_SRC_ALPHA, GL_ONE_MINUS_SRC_ALPHA);
    }

    let mut gl = GL::new();
    let mut renderer = render::Renderer::new();

    let text_shader = gl.create_camera_program(&camera_vert_glsl, &text_frag_glsl)?;
    let color_shader = gl.create_camera_program(&camera_vert_glsl, &color_frag_glsl)?;

    let ui_text_shader = gl.create_ui_program(&ui_vert_glsl, &text_frag_glsl)?;
    let ui_color_shader = gl.create_ui_program(&ui_vert_glsl, &color_frag_glsl)?;

    let font_atlas = FontAtlas::new("iosevka.ttf")?;

    let mut camera = Camera::new();
    let mut cursor = Cursor::new(V4::rgba(1.0, 1.0, 1.0, 0.5));

    let cursor_size = V2 {
        x: font_atlas.glyph('?').ax,
        y: -(FONT_PIXEL_HEIGHT as f32),
    };

    'main_loop: loop {
        let start = sdl.get_ticks();

        while let Some((event, _ts)) = sdl.poll_events() {
            match event {
                events::Event::Quit => break 'main_loop,
                events::Event::TextInput {
                    win_id: _,
                    text: input,
                } if editor.mode == Mode::Normal => {
                    if editor.handle_normal(&input) {
                        cursor.active();
                    }
                }
                events::Event::TextInput {
                    win_id: _,
                    text: input,
                } if editor.mode == Mode::Insert => {
                    editor.insert(&input);
                    cursor.active();
                }
                events::Event::TextInput {
                    win_id: _,
                    text: input,
                } if editor.mode == Mode::Command => {
                    editor.handle_command(&input);
                }
                events::Event::Key {
                    win_id: _,
                    pressed: true,
                    repeat: _,
                    scancode: _,
                    keycode,
                    modifiers: _,
                } => match keycode {
                    fermium::keycode::SDLK_ESCAPE if editor.mode == Mode::Insert => {
                        editor.exit_insert();
                        cursor.active();
                    }
                    fermium::keycode::SDLK_ESCAPE if editor.mode == Mode::Command => {
                        editor.exit_command();
                    }
                    fermium::keycode::SDLK_BACKSPACE if editor.mode == Mode::Insert => {
                        editor.delete();
                        cursor.active();
                    }
                    fermium::keycode::SDLK_BACKSPACE if editor.mode == Mode::Command => {
                        editor.command_delete_char();
                    }
                    fermium::keycode::SDLK_RETURN if editor.mode == Mode::Insert => {
                        editor.new_line();
                        cursor.active();
                    }
                    _ => (),
                },
                _ => (),
            }
        }

        cursor.update(DELTA_TIME);
        camera.update(DELTA_TIME);
        editor.update(DELTA_TIME);

        let (win_width, win_height) = win.get_window_size().into();
        let resolution = (win_width, win_height).into();

        unsafe {
            // TODO: necessary all the time?
            glViewport(0, 0, win_width, win_width);
            glClearColor(0.1, 0.1, 0.1, 1.0);
            glClear(GL_COLOR_BUFFER_BIT);
        }

        let mut max_line_length = 0.0f32;

        // render text
        {
            text_shader.activate(&resolution, &camera);

            let text_color = V4::rgb(1.0, 1.0, 0.1);

            let mut y_offset = 0.0;

            for line in editor.iter() {
                let mut x_offset = 0.0;

                for word in line {
                    x_offset += renderer.render_text(
                        &font_atlas,
                        word,
                        (x_offset, y_offset).into(),
                        text_color,
                        1.0,
                    );
                }

                y_offset -= FONT_PIXEL_HEIGHT as f32;
                max_line_length = max_line_length.max(x_offset);
            }
            renderer.flush();
        }

        // render cursor
        {
            let cursor_target = (editor.cursor() + (0.0, CURSOR_OFFSET).into()) * cursor_size;
            cursor.move_to(cursor_target);

            camera.target(cursor.pos, max_line_length, win_width as f32);

            if cursor.visible() {
                color_shader.activate(&resolution, &camera);
                cursor.render(&mut renderer, editor.mode != Mode::Insert);
                renderer.flush();
            }
        }

        // render UI
        {
            ui_color_shader.activate(&resolution);

            renderer.render_solid_rect(
                (-resolution.x / 2.0, -resolution.y / 2.0).into(),
                (resolution.x, 25.0).into(),
                V4::rgb(0.5, 0.8, 0.5),
            );
            renderer.flush();

            ui_text_shader.activate(&resolution);

            renderer.render_text(
                &font_atlas,
                editor.status_line(),
                (-resolution.x / 2.0, -resolution.y / 2.0).into(),
                V4::rgb(0.0, 0.0, 0.0),
                0.1,
            );

            renderer.flush();
        }

        win.swap_window();

        let finished = sdl.get_ticks();
        let duration = finished - start;

        if duration < DELTA_TIME_MS {
            unsafe {
                SDL_Delay(DELTA_TIME_MS - duration);
            }
        }
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
