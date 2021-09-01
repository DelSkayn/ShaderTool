#![allow(dead_code)]

#[macro_use]
extern crate anyhow;
#[macro_use]
extern crate log;
use anyhow::Result;
use glium::glutin::event_loop::EventLoop;

mod app;
mod asset;
mod config;
mod geom;
mod gui;
mod render;
mod util;

fn main() -> Result<()> {
    env_logger::init();

    let event_loop = EventLoop::<app::UserEvent>::with_user_event();
    let mut app = app::App::new(&event_loop)?;

    event_loop.run(move |event, _, control_flow| {
        app.handle_event(event, control_flow);
    });
}
