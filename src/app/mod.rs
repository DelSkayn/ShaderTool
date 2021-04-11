use crate::gui;
use anyhow::Result;
use glam::f32::*;
use glium::{implement_vertex, Display, Frame, Surface};
use glutin::{
    event::{DeviceEvent, MouseScrollDelta, WindowEvent},
    event_loop::EventLoop,
};
use notify::{
    event::{AccessKind, AccessMode, Event as NotifyEvent},
    immediate_watcher, EventKind, RecommendedWatcher, Result as NotifyResult, Watcher,
};
use std::path::PathBuf;

mod resource;
pub use resource::{Resource, ResourceId, Resources};
mod config;

mod camera;
use camera::CaptureMouse;

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
    cursor_grapped: bool,
}

impl Application {
    pub fn new(event_loop: &EventLoop<UserEvent>, display: &Display) -> Result<Self> {
        let mut resources = Resources::new();
        let config = resources.insert::<config::LoadedConfig, _>("./ShaderTool.json", display)?;
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

        let conf = resources.get_mut(&config).unwrap();
        conf.camera_handler
            .handle_mouse_move(&mut conf.camera, Vec2::ZERO);

        Ok(Application {
            resources,
            should_run: true,
            gui_state: gui::State::new(),
            watcher,
            config,
            cursor_grapped: false,
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

    pub fn update(&mut self, display: &Display) {
        if self.cursor_grapped {
            let window = display.gl_window();
            let window = window.window();
            let mut size = window.inner_size();
            size.width /= 2;
            size.height /= 2;
            let postion = glutin::dpi::PhysicalPosition::new(size.width, size.height);
            window.set_cursor_position(postion).ok();
        }
    }

    pub fn handle_window_event(
        &mut self,
        event: WindowEvent,
        display: &Display,
        over_window: bool,
    ) {
        let config = self.resources.get_mut(&self.config).unwrap();

        let mut capture = None;

        match event {
            WindowEvent::MouseInput { state, button, .. } => {
                if !over_window {
                    capture =
                        config
                            .camera_handler
                            .handle_mouse_click(&mut config.camera, button, state);
                }
            }
            _ => {}
        }

        if let Some(capture) = capture {
            if capture == CaptureMouse::Capture {
                display.gl_window().window().set_cursor_grab(true).ok();
                display.gl_window().window().set_cursor_visible(false);
                self.cursor_grapped = true;
            }

            if capture == CaptureMouse::Release {
                display.gl_window().window().set_cursor_grab(false).ok();
                display.gl_window().window().set_cursor_visible(true);
                self.cursor_grapped = false;
            }
        }
    }

    pub fn handle_device_event(
        &mut self,
        event: DeviceEvent,
        display: &Display,
        over_window: bool,
    ) {
        if over_window {
            return;
        }

        let config = self.resources.get_mut(&self.config).unwrap();

        let mut capture = None;

        match event {
            DeviceEvent::MouseMotion { delta } => {
                let delta = Vec2::new(delta.0 as f32, delta.1 as f32);
                capture = config
                    .camera_handler
                    .handle_mouse_move(&mut config.camera, delta);
            }
            DeviceEvent::MouseWheel { delta } => {
                let delta = match delta {
                    MouseScrollDelta::LineDelta(x, y) => Vec2::new(x, y) * 12.0,
                    MouseScrollDelta::PixelDelta(x) => {
                        let logical = x
                            .to_logical::<f64>(display.gl_window().window().scale_factor())
                            .cast::<f32>();
                        Vec2::new(logical.x, logical.y)
                    }
                };
                config
                    .camera_handler
                    .handle_mouse_scroll(&mut config.camera, delta);
            }
            _ => {}
        }

        if let Some(capture) = capture {
            if capture == CaptureMouse::Capture {
                display.gl_window().window().set_cursor_grab(true).ok();
                display.gl_window().window().set_cursor_visible(false);
                self.cursor_grapped = true;
            }

            if capture == CaptureMouse::Release {
                display.gl_window().window().set_cursor_grab(false).ok();
                display.gl_window().window().set_cursor_visible(true);
                self.cursor_grapped = false;
            }
        }
    }

    pub fn render(&self, frame: &mut Frame) -> Result<()> {
        let config = self.resources.get(&self.config).unwrap();
        let dim = frame.get_dimensions();
        let fov = dim.0 as f32 / dim.1 as f32;

        let view = Mat4::from_quat(config.camera.rotation.conjugate())
            * Mat4::from_translation(-config.camera.position);
        //let view = Mat4::from_translation(-config.camera.position) * Mat4::from_quat(config.camera.rotation.conjugate()) ;

        let data = RenderData {
            view,
            projection: Mat4::perspective_lh(config.camera.fov.to_radians(), fov, 0.1, 1000.0),
        };

        for p in config.passes.iter() {
            p.render(frame, &data)?;
        }
        Ok(())
    }
}
