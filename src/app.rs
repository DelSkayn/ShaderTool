use crate::{
    asset::{self, AssetRef},
    config::Config,
    gui::Model,
};
use anyhow::{Context, Result};
use egui_glium::EguiGlium;
use glium::{
    glutin::{
        self,
        event::Event,
        event_loop::{ControlFlow, EventLoop, EventLoopProxy},
        window::WindowBuilder,
    },
    Display,
};
use notify::{
    event::{AccessKind, AccessMode, Event as NotifyEvent},
    EventKind, RecommendedWatcher, Result as NotifyResult, Watcher,
};
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub enum UserEvent {
    FileChanged(PathBuf),
}

pub struct App {
    ron: bool,
    old_config: Option<AssetRef<Config>>,
    config: Option<AssetRef<Config>>,
    display: Display,
    egui: EguiGlium,
    _watcher: RecommendedWatcher,
    model: Model,
}

impl App {
    pub fn new(event_loop: &EventLoop<UserEvent>) -> Result<Self> {
        // Load the config file
        let ron_exists = Path::new("./ShaderTool.ron").exists();
        let json_exists = Path::new("./ShaderTool.json").exists();
        if !ron_exists && !json_exists {
            bail!("Could not find config file, create either a `ShaderTool.ron` or a `ShaderTool.json` file in the current directory");
        } else if json_exists && ron_exists {
            warn!(
                "Both a ShaderTool.ron and ShaderTool.json exist in this directory! defaulting to ron"
            );
        }

        // Create display and setup egui
        let display = Self::create_display(event_loop).context("failed to create an window")?;
        let egui = EguiGlium::new(&display);
        let _watcher = Self::create_watcher(event_loop.create_proxy())
            .context("could not create a file watcher")?;

        let mut model = Model::new();

        let config = if ron_exists {
            asset::AssetRef::build(Config::load, "./ShaderTool.ron", &display)
        } else {
            asset::AssetRef::build(Config::load, "./ShaderTool.json", &display)
        };

        let config = config
            .map_err(|e| {
                error!("{:?}", e);
                model.set_error(Some(format!("{:?}", e)));
                e
            })
            .ok();

        Ok(App {
            egui,
            ron: ron_exists,
            display,
            _watcher,
            old_config: None,
            config,
            model,
        })
    }

    fn create_display(event_loop: &EventLoop<UserEvent>) -> Result<Display> {
        let window_builder = WindowBuilder::new()
            .with_resizable(true)
            .with_title("Shader tool");

        let context_builder = glutin::ContextBuilder::new()
            .with_depth_buffer(8)
            .with_srgb(true)
            .with_stencil_buffer(0)
            .with_vsync(true);

        Ok(Display::new(window_builder, context_builder, event_loop)?)
    }

    fn create_watcher(proxy: EventLoopProxy<UserEvent>) -> Result<RecommendedWatcher> {
        let mut watcher = notify::recommended_watcher(move |ev: NotifyResult<NotifyEvent>| {
            if let Ok(x) = ev {
                if x.kind != EventKind::Access(AccessKind::Close(AccessMode::Write)) {
                    return;
                }
                for p in x.paths {
                    if let Ok(x) = p.canonicalize() {
                        proxy.send_event(UserEvent::FileChanged(x)).ok();
                    }
                }
            }
        })?;
        watcher.watch(Path::new("./"), notify::RecursiveMode::Recursive)?;
        Ok(watcher)
    }

    fn redraw(&mut self, control_flow: &mut ControlFlow) -> Result<()> {
        self.egui.begin_frame(&self.display);

        self.model.draw(self.egui.ctx());

        let (needs_repaint, shapes) = self.egui.end_frame(&self.display);

        *control_flow = if !self.model.should_run() {
            glutin::event_loop::ControlFlow::Exit
        } else if needs_repaint {
            self.display.gl_window().window().request_redraw();
            glutin::event_loop::ControlFlow::Poll
        } else {
            glutin::event_loop::ControlFlow::Wait
        };

        {
            use glium::Surface as _;
            let mut target = self.display.draw();

            let clear_color = egui::Rgba::from_rgb(0.1, 0.3, 0.2);
            target.clear_color_and_depth(
                (
                    clear_color[0],
                    clear_color[1],
                    clear_color[2],
                    clear_color[3],
                ),
                1.0,
            );

            if let Some(x) = self.config.as_ref().map(|x| x.borrow()) {
                match x.render(&mut target) {
                    Ok(_) => {}
                    Err(e) => {
                        target.finish().ok();
                        return Err(e);
                    }
                }
            }

            self.egui.paint(&self.display, &mut target, shapes);

            target.finish()?;
            Ok(())
        }
    }

    pub fn handle_event(&mut self, event: Event<UserEvent>, control_flow: &mut ControlFlow) {
        match event {
            // Platform-dependent event handlers to workaround a winit bug
            // See: https://github.com/rust-windowing/winit/issues/987
            // See: https://github.com/rust-windowing/winit/issues/1619
            Event::RedrawEventsCleared if cfg!(windows) => match self.redraw(control_flow) {
                Ok(_) => {
                    self.old_config.take();
                }
                Err(e) => {
                    error!("{:?}", e);
                    self.config = self.old_config.take();
                    self.model.set_error(Some(format!("{:?}", e)));
                    *control_flow = ControlFlow::Poll;
                }
            },
            Event::RedrawRequested(_) if !cfg!(windows) => match self.redraw(control_flow) {
                Ok(_) => {
                    self.old_config.take();
                }
                Err(e) => {
                    error!("{:?}", e);
                    self.config = self.old_config.take();
                    self.model.set_error(Some(format!("{:?}", e)));
                    *control_flow = ControlFlow::Poll;
                }
            },

            Event::WindowEvent { event, .. } => {
                if let Some(mut x) = self.config.as_ref().map(|x| x.borrow_mut()) {
                    x.handle_window_event(&event)
                }

                if self.egui.is_quit_event(&event) {
                    *control_flow = glium::glutin::event_loop::ControlFlow::Exit;
                }

                self.egui.on_event(&event);

                self.display.gl_window().window().request_redraw(); // TODO: ask egui if the events warrants a repaint instead
            }
            Event::DeviceEvent { event, .. } => {
                if let Some(mut x) = self.config.as_ref().map(|x| x.borrow_mut()) {
                    x.handle_device_event(&event)
                }
            }
            Event::UserEvent(UserEvent::FileChanged(path)) => {
                if self.config.is_none() {
                    let config = if self.ron {
                        AssetRef::build(Config::load, "./ShaderTool.ron", &self.display)
                    } else {
                        AssetRef::build(Config::load, "./ShaderTool.json", &self.display)
                    };
                    info!("trying to reload config");

                    match config {
                        Ok(x) => {
                            self.config = Some(x);
                            self.model.set_error(None);
                        }
                        Err(e) => {
                            error!("{:?}", e);
                            self.model.set_error(Some(format!("{:?}", e)));
                        }
                    }
                } else {
                    match asset::reload(&path) {
                        Err(e) => {
                            error!("{:?}", e);
                            self.model.set_error(Some(format!("{:?}", e)));
                        }
                        Ok(()) => self.model.set_error(None),
                    }
                }
                self.display.gl_window().window().request_redraw();
            }
            _ => (),
        }
    }
}
