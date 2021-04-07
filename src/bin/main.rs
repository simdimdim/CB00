use cb00::{
    app::App,
    common::{Draw, Prepare},
};
use graphics::clear;
use piston_window::{
    AdvancedWindow,
    Button,
    EventLoop,
    IdleEvent,
    Key,
    MouseCursorEvent,
    MouseScrollEvent,
    PistonWindow,
    PressEvent,
    RenderEvent,
    ResizeEvent,
    UpdateEvent,
};
use sdl2::video::FullscreenType;
use sdl2_window::Sdl2Window;

// #[tokio::main(flavor = "current_thread")]
#[tokio::main]
async fn main() { run().await }

async fn run() {
    let mut app = App::default();
    app.add_folder(std::env::args().nth(1).unwrap_or_else(|| ".".to_string()));
    // app.test().await;
    let mut window: PistonWindow<Sdl2Window> =
        app.settings.window.build().unwrap();
    window.set_capture_cursor(app.settings.capture);
    window.set_max_fps(app.settings.fps);
    window.set_ups(app.settings.ups);
    let mut cursor = [0.; 2];
    //main loop
    while let Some(e) = window.next() {
        let mut ctx = window.create_texture_context();
        app.prepare(&mut ctx);

        window.draw_2d(&e, |c, g, _device| {
            clear([0.0; 4], g);
            app.draw(c, g, None);
            // sorted.sort_by(|(a, _), (b, _)| {
            //     a.file_name().partial_cmp(&b.file_name()).unwrap()
            // });
        });
        if let Some(_) = e.resize_args() {
            app.resize(&window);
        }
        if let Some(pos) = e.mouse_cursor(|xy| xy) {
            cursor = pos;
        };
        e.mouse_scroll(|d| {
            d[1];
        });
        if let Some(button) = e.press_args() {
            if let Button::Keyboard(key) = button {
                match key {
                    Key::A | Key::Left => app.prev_page(),
                    Key::D | Key::Right => app.next_page(),
                    Key::W | Key::Up | Key::NumPadPlus => app.more(),
                    Key::S | Key::Down | Key::NumPadMinus => app.less(),
                    Key::R => app.toggle_direction(),
                    Key::Q => break,
                    Key::F | Key::F12 => {
                        match window.window.window.fullscreen_state() {
                            FullscreenType::Off => &window
                                .window
                                .window
                                .set_fullscreen(FullscreenType::Desktop),
                            FullscreenType::True => &window
                                .window
                                .window
                                .set_fullscreen(FullscreenType::Desktop),
                            FullscreenType::Desktop => &window
                                .window
                                .window
                                .set_fullscreen(FullscreenType::Off),
                        };
                    }
                    _ => {}
                }
            }
        }
        if let Some(_args) = e.idle_args() {
            // println!("{}", args.dt);
        }
        if let Some(_args) = e.render_args() {
            // app.render(&args);
        }
        if let Some(_args) = e.update_args() {
            // println!("{}", args.dt);
            // app.update();
        }
    }
    #[allow(path_statements)]
    {
        cursor;
        app.ar;
    }
}
