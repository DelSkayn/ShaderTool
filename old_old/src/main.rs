#![allow(dead_code)]

#[macro_use]
extern crate log;

use anyhow::{Result,anyhow,Context};
use std::path::{PathBuf,Path};

use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

#[derive(Debug)]
pub enum UserEvent {
    Reload(PathBuf),
    Gui(gui::Event),
}

mod gui;
mod renderer;
mod watcher;
mod resources;
mod config;
use config::LoadedConfig;

pub struct State{
    renderer: renderer::Renderer,
    compiler: shaderc::Compiler,
}

fn main() -> Result<()> {
    env_logger::init();

    let compiler = shaderc::Compiler::new().ok_or(anyhow!("failed to initialize the shader compiler"))?;
    let event_loop = EventLoop::<UserEvent>::with_user_event();
    let proxy = event_loop.create_proxy();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    let _watcher = watcher::build(proxy)?;

    let mut imgui = imgui::Context::create();
    imgui.set_ini_filename(None);
    let mut platform = imgui_winit_support::WinitPlatform::init(&mut imgui);
    platform.attach_window(
        imgui.io_mut(),
        &window,
        imgui_winit_support::HiDpiMode::Default,
    );

    let mut resources = resources::Resources::new();

    let renderer = futures::executor::block_on(renderer::Renderer::new(&window, &mut imgui))?;
    let mut gui_state = gui::State::new(event_loop.create_proxy());
    let mut state = State{
        renderer,
        compiler,
    };

    let config_load = if Path::new("./ShaderTool.json5").exists(){
        resources.insert::<LoadedConfig,_>("./ShaderTool.json5",(),&mut state)
    }else{
        resources.insert::<LoadedConfig,_>("./ShaderTool.json",(),&mut state)
    };

    if let Err(e) = config_load{
        error!("{:?}",e);
        gui_state.set_error(format!("{:?}",e));
    }

    event_loop.run(move |event, _, control_flow| {
        platform.handle_event(imgui.io_mut(), &window, &event);
        match event {
            Event::MainEventsCleared => {
                platform.prepare_frame(imgui.io_mut(), &window)
                    .expect("Failed to prepare frame");
                window.request_redraw();
            }
            Event::WindowEvent {
                ref event,
                ..
            } => match event {
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                WindowEvent::Resized(physical_size) => {
                    state.renderer.resize(*physical_size);
                }
                WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                    state.renderer.resize(**new_inner_size);
                }
                WindowEvent::KeyboardInput { input, .. } => match input {
                    winit::event::KeyboardInput {
                    state: winit::event::ElementState::Pressed,
                    virtual_keycode: Some(winit::event::VirtualKeyCode::Escape),
                    ..
                    } => {gui_state.set_error("BLA")}
                    _ => {},
                }
                _ => {}
            },
            Event::UserEvent(event) => {
                match event {
                    UserEvent::Gui(event) => {
                        match event {
                            gui::Event::Quit => { *control_flow = ControlFlow::Exit },
                        }
                    },
                    UserEvent::Reload(path) => {
                        if let Err(e) = resources.reload(&path,&mut state)
                            .with_context(|| format!("Failed to reload path: {}",path.display()))
                        {
                            error!("{:?}",e);
                            gui_state.set_error(format!("{:?}",e))
                        }
                    }
                }
            }
            Event::RedrawRequested(_) => {
                let ui_frame = imgui.frame();

                gui::render(&mut gui_state, &ui_frame, &window);

                platform.prepare_render(&ui_frame,&window);

                state.renderer.render(ui_frame.render()).unwrap();
            }
            _ => {}
        }
    })
}
