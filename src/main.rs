#![allow(dead_code)]

use anyhow::Result;
use std::path::PathBuf;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

#[derive(Debug)]
pub enum UserEvent {
    Reload(PathBuf),
}

mod renderer;
mod watcher;

fn main() -> Result<()> {
    env_logger::init();

    let event_loop = EventLoop::<UserEvent>::with_user_event();
    let proxy = event_loop.create_proxy();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    let _watcher = watcher::build(proxy)?;

    let imgui = imgui::Context::create();

    let _renderer = futures::executor::block_on(renderer::Renderer::new(&window, &mut imgui))?;

    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() => {
                match event {
                    WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                    /*
                    WindowEvent::KeyboardInput { input, .. } => match input {
                    KeyboardInput {
                    state: ElementState::Pressed,
                    virtual_keycode: Some(VirtualKeyCode::Escape),
                    ..
                    } => *control_flow = ControlFlow::Exit,
                    _ => {}
                    },
                    WindowEvent::Resized(physical_size) => {
                    state.resize(*physical_size);
                    }
                    WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                    // new_inner_size is &mut so w have to dereference it twice
                    state.resize(**new_inner_size);
                    }
                                            */
                    _ => {}
                }
            }
            _ => {}
        }
    })
}
