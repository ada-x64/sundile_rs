mod debug_gui;
mod game;
// mod input;
mod renderer;
mod renderer2d;

use winit::{event::*, event_loop::*, window::*};
use std::time::*;

use sundile_graphics::prelude::*;

pub fn run(bin: &[u8]) {
    println!("===\n=== RUN AT {:?}\n===", chrono::Local::now());

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_maximized(true)
        .with_resizable(false)
        .with_transparent(false)
        .with_decorations(true) 
        //TODO p4: .with_icon()
        .build(&event_loop).unwrap();

    let mut render_target = futures::executor::block_on(RenderTarget::new(&window, false, Some("Renderer")));

    let mut gui = debug_gui::DebugGui::new(&render_target, &window);
    let mut game = game::Game::new(&render_target, bin, None);
    game.init_scene(0);

    let mut view_debug_gui = false;
    let mut fps = 0.0;
    let mut prev_time = Instant::now();
 
    event_loop.run(move |event, _, control_flow| {

        gui.handle_event(&event);

        match event {
            winit::event::Event::MainEventsCleared => {
                        
                let time = Instant::now();
                let dt = time - prev_time;
                prev_time = time;

                let smoothing = 0.9;
                fps = fps*smoothing + (1.0-smoothing)/dt.as_secs_f64();
                if !view_debug_gui {
                    game.update(dt);
                }
                
                render_target.begin_frame();
                //TODO: Thread this process
                game.render(&mut render_target,);
                if view_debug_gui {
                    gui.render(&mut render_target, &window, &game);
                }
                render_target.end_frame();
            },
            winit::event::Event::WindowEvent {window_id, event}
                if window_id == window.id() => {
                match event {
                    WindowEvent::CloseRequested => {
                        *control_flow = ControlFlow::Exit;
                    },
                    WindowEvent::KeyboardInput {input, ..} => {
                        if input.state == ElementState::Released {
                            match input.virtual_keycode {
                                Some(code) => {
                                    match code {
                                        VirtualKeyCode::Escape => {
                                            view_debug_gui = !view_debug_gui;
                                        }
                                        _ => {}
                                    }
                                }
                                None => {}
                            }
                        }
                    },
                    _ => {
                    }
                }
            },
            winit::event::Event::DeviceEvent {ref event, ..} => {
                if !view_debug_gui {
                    game.handle_input(&event);
                }
            },
            _ => {}
        }
    });
}