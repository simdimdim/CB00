#![feature(destructuring_assignment)]
#![feature(exclusive_range_pattern)]

use cb00::{
    app::App,
    element::{Element, Picture},
    Assets,
};
use gfx_device_gl::{CommandBuffer, Resources};
use gfx_graphics::GfxGraphics;
use graphics::{clear, Context, Transformed};
use piston_window::{
    image,
    AdvancedWindow,
    Button,
    EventLoop,
    G2dTextureContext,
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
use std::cmp::max;

#[tokio::main]
async fn main() {
    let mut app = App::default();
    app.test().await;
    return;
    let mut window: PistonWindow<Sdl2Window> =
        app.settings.window.build().unwrap();
    window.set_capture_cursor(app.settings.capture);
    window.set_max_fps(app.settings.fps);
    window.set_ups(app.settings.ups);
    let mut cursor = [0.; 2];
    let mut assets = Assets::default();

    while let Some(e) = window.next() {
        let mut ctx = window.create_texture_context();
        window.draw_2d(&e, |c, g, _device| {
            clear([0.0; 4], g);
            let mut offsetx = 0u32;
            let mut offsety = (0u32, 0u32);
            let mut sorted: Vec<_> = assets.elements.iter_mut().collect();
            sorted.sort_by(|(a, _), (b, _)| {
                a.file_name().partial_cmp(&b.file_name()).unwrap()
            });
            for (_, el) in sorted {
                // draw(
                //     el,
                //     &mut ctx,
                //     c,
                //     g,
                //     &mut offsetx,
                //     &mut offsety,
                //     &app.width,
                //     &app.height,
                // );
            }
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
            assets.list_files();
        }
    }
    #[allow(path_statements)]
    {
        cursor;
        app.ar;
    }
}
fn draw(
    el: &mut Element,
    ctx: &mut G2dTextureContext,
    c: Context,
    g: &mut GfxGraphics<Resources, CommandBuffer>,
    offsetx: &mut u32,
    offsety: &mut (u32, u32),
    width: &f64,
    _height: &f64,
) {
    el.t(ctx);
    match el {
        Element::File(_, Some(Picture { w, h, tex, .. })) => {
            const ZOOM: f64 = 0.245;
            let transform = c
                .trans(*offsetx as f64, offsety.0 as f64)
                .transform
                .append_transform(graphics::math::scale(ZOOM, ZOOM));
            // dbg!(c.transform);
            if let Some(texture) = tex {
                image(texture, transform, g);
            }
            let new_offsetx = (ZOOM * *w as f64) as u32;
            if ((*offsetx + new_offsetx) as f64) < *width - new_offsetx as f64 {
                *offsetx += new_offsetx;
                offsety.1 = max(
                    (*h as f64 * ZOOM) as u32,
                    (offsety.1 as f64 * ZOOM) as u32,
                );
            } else {
                // dbg!(&offsety);
                offsety.0 += offsety.1;
                offsety.1 = 0;
                *offsetx = 0;
            }
        }
        _ => {}
    }
    // println!("{}", el);
}
