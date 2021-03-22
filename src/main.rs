#![allow(dead_code)]

#[macro_use]
extern crate log;

use anyhow::Result;
use glium::Surface;
use glutin::{
    event::{Event, StartCause, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use imgui::Context as ImContext;
use imgui_glium_renderer::Renderer;
use imgui_winit_support::{HiDpiMode, WinitPlatform};
use std::time::{Duration, Instant};

mod app;
mod gui;
pub use app::Application;

fn main() -> Result<()> {
    env_logger::init();
    info!("starting shader tool");
    let event_loop = EventLoop::with_user_event();
    let wb = WindowBuilder::new().with_title("Shader Tool");
    let cb = glium::glutin::ContextBuilder::new();
    let display = glium::Display::new(wb, cb, &event_loop)?;

    let mut imgui = ImContext::create();
    imgui.set_ini_filename(None);
    let mut imgui_renderer = Renderer::init(&mut imgui, &display)?;

    let mut platform = WinitPlatform::init(&mut imgui);
    platform.attach_window(
        imgui.io_mut(),
        &display.gl_window().window(),
        HiDpiMode::Default,
    );

    let mut app = Application::new(&event_loop, &display)?;

    let mut last_frame = Instant::now();

    event_loop.run(move |event, _, control_flow| {
        platform.handle_event(imgui.io_mut(), display.gl_window().window(), &event);
        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                *control_flow = ControlFlow::Exit;
            }
            Event::WindowEvent { event, .. } => {
                let io = imgui.io_mut();
                if io.want_capture_mouse || io.want_capture_keyboard {
                    return;
                }
                app.handle_window_event(event, &display);
            }
            Event::RedrawRequested(_) => {
                platform
                    .prepare_frame(imgui.io_mut(), display.gl_window().window())
                    .expect("failed to prepare frame");
                let ui = imgui.frame();

                gui::build(&mut app, &ui, &display);

                platform.prepare_render(&ui, display.gl_window().window());
                let mut frame = display.draw();
                frame.clear_color_and_depth((0.0, 0.0, 0.0, 1.0), 1.0);
                app.render(&mut frame).unwrap();

                let draw_data = ui.render();

                imgui_renderer.render(&mut frame, &draw_data).unwrap();

                frame.finish().unwrap();

                *control_flow =
                    ControlFlow::WaitUntil(last_frame + Duration::from_secs_f32(1.0 / 60.0));
                last_frame = Instant::now();

                if !app.should_run {
                    *control_flow = ControlFlow::Exit;
                }
            }
            Event::NewEvents(StartCause::WaitCancelled {
                requested_resume, ..
            }) => {
                *control_flow = ControlFlow::WaitUntil(requested_resume.unwrap());
            }
            Event::NewEvents(StartCause::ResumeTimeReached { .. }) => {
                display.gl_window().window().request_redraw();
            }
            Event::NewEvents(StartCause::Init) => {
                display.gl_window().window().request_redraw();
            }
            Event::UserEvent(x) => app.handle_event(x, &display),
            _ => {}
        }
    });
}
