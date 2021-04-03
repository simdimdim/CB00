#![feature(destructuring_assignment)]
#![feature(exclusive_range_pattern)]

use cb00::{app::App, common::Draw};

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
use sdl2_window::Sdl2Window;

#[tokio::main]
async fn main() {
    let mut app = App::default();
    app.test().await;
    let mut window: PistonWindow<Sdl2Window> =
        app.settings.window.build().unwrap();
    window.set_capture_cursor(app.settings.capture);
    window.set_max_fps(app.settings.fps);
    window.set_ups(app.settings.ups);
    let mut cursor = [0.; 2];

    while let Some(e) = window.next() {
        let mut ctx = window.create_texture_context();
        window.draw_2d(&e, |c, g, _device| {
            clear([0.0; 4], g);
            app.draw(&mut ctx, c, g);
            // let mut offsetx = 0u32;
            // let mut offsety = (0u32, 0u32);
            // let mut sorted: Vec<_> = assets.elements.iter_mut().collect();
            // sorted.sort_by(|(a, _), (b, _)| {
            //     a.file_name().partial_cmp(&b.file_name()).unwrap()
            // });
            // for (_, el) in sorted {
            //     // draw(
            //     //     el,
            //     //     &mut ctx,
            //     //     c,
            //     //     g,
            //     //     &mut offsetx,
            //     //     &mut offsety,
            //     //     &app.width,
            //     //     &app.height,
            //     // );
            // }
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
                    Key::Q => {
                        break;
                    }
                    Key::F => {}
                    Key::A => {}
                    Key::D => {}
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
