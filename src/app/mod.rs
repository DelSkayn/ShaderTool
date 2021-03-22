use crate::gui;
use anyhow::Result;
use glam::f32::*;
use glium::{implement_vertex, Display, Frame, Surface};
use glutin::{event::WindowEvent, event_loop::EventLoop};
use notify::{
    event::{AccessKind, AccessMode, Event as NotifyEvent},
    immediate_watcher, EventKind, RecommendedWatcher, Result as NotifyResult, Watcher,
};
use std::path::PathBuf;

mod resource;
pub use resource::{AnyResourceId, Resource, ResourceId, Resources};
mod config;

mod camera;

//mod pass;

pub struct RenderData {
    view: Mat4,
    projection: Mat4,
}

#[derive(Clone, Copy)]
pub struct Vertex {
    position: [f32; 3],
    normal: [f32; 3],
    tex_coord: [f32; 2],
}

implement_vertex!(Vertex, position, normal, tex_coord);

#[derive(Debug)]
pub enum UserEvent {
    FileChanged(PathBuf),
}

pub struct Application {
    pub should_run: bool,
    pub gui_state: gui::State,
    config: ResourceId<config::LoadedConfig>,
    resources: Resources,
    watcher: RecommendedWatcher,
}

impl Application {
    pub fn new(event_loop: &EventLoop<UserEvent>, display: &Display) -> Result<Self> {
        let mut resources = Resources::new();
        let config = resources.insert("./ShaderTool.json", display)?;
        let event_loop = event_loop.create_proxy();
        let mut watcher = immediate_watcher(move |ev: NotifyResult<NotifyEvent>| {
            if let Ok(x) = ev {
                if x.kind != EventKind::Access(AccessKind::Close(AccessMode::Write)) {
                    return;
                }
                for p in x.paths {
                    if let Ok(x) = p.canonicalize() {
                        event_loop.send_event(UserEvent::FileChanged(x)).ok();
                    }
                }
            }
        })?;
        watcher.watch("./", notify::RecursiveMode::Recursive)?;
        Ok(Application {
            resources,
            should_run: true,
            gui_state: gui::State::new(),
            watcher,
            config,
        })
    }

    pub fn handle_event(&mut self, event: UserEvent, display: &Display) {
        match event {
            UserEvent::FileChanged(x) => {
                match self.resources.reload(x, display) {
                    Err(e) => {
                        error!("Error reloading:\n {:?}", e);
                        self.gui_state.set_error(format!("{:?}", e));
                    }
                    Ok(true) => {
                        self.gui_state.clear_error();
                    }
                    Ok(false) => {}
                };
            }
        }
    }

    pub fn handle_window_event(&mut self, event: WindowEvent, display: &Display) {}

    pub fn render(&self, frame: &mut Frame) -> Result<()> {
        let config = self.resources.get(self.config).unwrap();
        let dim = frame.get_dimensions();
        let fov = dim.0 as f32 / dim.1 as f32;

        let data = RenderData {
            view: Mat4::look_at_lh(Vec3::new(2.0, 1.0, -5.0), Vec3::ZERO, Vec3::Y),
            projection: Mat4::perspective_lh(90.0f32.to_radians(), fov, 0.1, 1000.0),
        };

        for p in config.passes.iter() {
            p.render(frame, &data)?;
        }
        Ok(())
    }
}
