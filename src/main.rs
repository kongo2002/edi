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
use self::errors::EdiError;
use self::font::{FontAtlas, FONT_PIXEL_HEIGHT};
use self::render::{V2, V4};

mod camera;
mod errors;
mod font;
mod gl;
mod render;

const FPS: u32 = 60;
const DELTA_TIME_MS: u32 = 1000 / FPS;

#[derive(PartialEq)]
enum CooldownState {
    Active,
    OnCooldown,
}

struct Cooldown {
    cooldown: f32,
    duration: f32,
    current: f32,
    state: CooldownState,
}

impl Cooldown {
    pub fn new(duration: f32, cooldown: f32) -> Cooldown {
        Cooldown {
            cooldown,
            duration,
            current: 0.0,
            state: CooldownState::Active,
        }
    }

    pub fn reset(&mut self, state: CooldownState) {
        self.state = state;
        match self.state {
            CooldownState::Active => self.current = self.duration,
            CooldownState::OnCooldown => self.current = self.cooldown,
        }
    }

    pub fn update(&mut self, delta: f32) {
        match self.state {
            CooldownState::Active => {
                if self.current > delta {
                    self.current -= delta;
                } else {
                    self.current = self.cooldown;
                    self.state = CooldownState::OnCooldown;
                }
            }
            CooldownState::OnCooldown => {
                if self.current > delta {
                    self.current -= delta;
                } else {
                    self.current = self.duration;
                    self.state = CooldownState::Active;
                }
            }
        }
    }
}

struct Cursor {
    pos: V2,
    cd: Cooldown,
}

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

    unsafe {
        load_global_gl(&|f_name| win.get_proc_address(f_name));

        glEnable(GL_BLEND);
        glEnable(GL_MULTISAMPLE);
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
        scale: 0.3,
        scale_velocity: 1.0,
    };

    let mut cursor = Cursor {
        pos: (0, 0).into(),
        cd: Cooldown::new(400.0, 600.0),
    };

    let mut lines = vec![String::new()];

    'main_loop: loop {
        let start = sdl.get_ticks();

        while let Some((event, _ts)) = sdl.poll_events() {
            match event {
                events::Event::Quit => break 'main_loop,
                events::Event::TextInput {
                    win_id: _,
                    text: input,
                } => {
                    lines.last_mut().map(|line| line.push_str(&input));
                    cursor.cd.reset(CooldownState::Active);
                }
                events::Event::Key {
                    win_id: _,
                    pressed: true,
                    repeat: _,
                    scancode: _,
                    keycode,
                    modifiers: _,
                } => match keycode {
                    fermium::keycode::SDLK_BACKSPACE => {
                        lines.last_mut().map(|line| line.pop());
                        cursor.cd.reset(CooldownState::Active);
                    }
                    fermium::keycode::SDLK_RETURN => {
                        lines.push(String::new());
                        cursor.cd.reset(CooldownState::Active);
                    }
                    _ => (),
                },

                _ => (),
            }
        }

        cursor.cd.update(DELTA_TIME_MS as f32);

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

            let mut y_offset = 0.0;
            for line in &lines {
                renderer.render_text(&font_atlas, &line, (0.0, y_offset).into(), text_color);
                y_offset -= FONT_PIXEL_HEIGHT as f32;
            }
            renderer.flush();
        }

        // render cursor
        if cursor.cd.state == CooldownState::Active {
            const CURSOR_OFFSET: f32 = 0.13;

            color_shader.activate(&resolution, &camera);

            let cursor_size = ((FONT_PIXEL_HEIGHT as f32) / 6.0, FONT_PIXEL_HEIGHT as f32);
            let cursor_color = V4 {
                x: 1.0,
                y: 0.3,
                z: 0.3,
                a: 1.0,
            };

            let line_idx = (lines.len() as f32) - 1.0;
            let line_width = lines
                .last()
                .map(|line| font_atlas.line_width(line))
                .unwrap_or(0.0);
            let cursor_pos = (
                line_width,
                (-(line_idx + CURSOR_OFFSET)) * (FONT_PIXEL_HEIGHT as f32),
            );

            renderer.render_solid_rect(cursor_pos.into(), cursor_size.into(), cursor_color);
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
