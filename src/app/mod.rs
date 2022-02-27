use crate::config::Config;
use anyhow::{Context, Result};
use egui_glium::EguiGlium;
use glium::{
    glutin::{
        self,
        event::{Event, WindowEvent},
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

mod gui;

#[derive(Debug)]
pub enum UserEvent {
    FileChanged(PathBuf),
}

pub enum ConfigKind {
    Ron,
    Json,
}

pub enum State {
    /// No config could be found
    NotLoaded { error: String },
    /// Config loaded without trouble but could still error on render.
    FirstFrame {
        old_config: Option<Box<Config>>,
        config: Box<Config>,
        kind: ConfigKind,
    },
    /// Config loaded without trouble and rendered without encountering an error
    Loaded {
        config: Box<Config>,
        kind: ConfigKind,
    },
    /// Old config still valid but new config encountered an error.
    ReloadError {
        config: Box<Config>,
        error: String,
        kind: ConfigKind,
    },
}

impl State {
    pub fn active_config(&self) -> Option<&Config> {
        match *self {
            State::Loaded { ref config, .. }
            | State::FirstFrame { ref config, .. }
            | State::ReloadError { ref config, .. } => Some(&*config),
            _ => None,
        }
    }

    pub fn active_config_mut(&mut self) -> Option<&mut Config> {
        match *self {
            State::Loaded { ref mut config, .. }
            | State::FirstFrame { ref mut config, .. }
            | State::ReloadError { ref mut config, .. } => Some(&mut *config),
            _ => None,
        }
    }

    pub fn take(&mut self) -> Self {
        std::mem::replace(
            self,
            State::NotLoaded {
                error: String::new(),
            },
        )
    }
}

pub struct App {
    state: State,
    display: Display,
    egui: EguiGlium,
    _watcher: RecommendedWatcher,
    should_run: bool,
    gui: gui::Gui,
}

impl App {
    pub fn new(event_loop: &EventLoop<UserEvent>) -> Result<Self> {
        // Load the config file

        // Create display and setup egui
        let display = Self::create_display(event_loop).context("failed to create an window")?;
        let egui = EguiGlium::new(&display);
        let _watcher = Self::create_watcher(event_loop.create_proxy())
            .context("could not create a file watcher")?;

        let state = Self::initial_load_config(&display);

        Ok(App {
            egui,
            display,
            _watcher,
            state,
            should_run: true,
            gui: gui::Gui::new(),
        })
    }

    fn initial_load_config(display: &Display) -> State {
        let ron_exists = Path::new("./ShaderTool.ron").exists();
        let json_exists = Path::new("./ShaderTool.json").exists();
        if json_exists && ron_exists {
            warn!(
                "Both a ShaderTool.ron and ShaderTool.json exist in this directory! Defaulting to ShaderTool.ron."
            );
        }
        if ron_exists {
            match Config::load("./ShaderTool.ron", &display) {
                Ok(x) => State::FirstFrame {
                    old_config: None,
                    config: Box::new(x),
                    kind: ConfigKind::Ron,
                },
                Err(e) => State::NotLoaded {
                    error: format!("{:?}", e),
                },
            }
        } else if json_exists {
            match Config::load("./ShaderTool.json", &display) {
                Ok(x) => State::FirstFrame {
                    old_config: None,
                    config: Box::new(x),
                    kind: ConfigKind::Json,
                },
                Err(e) => State::NotLoaded {
                    error: format!("{:?}", e),
                },
            }
        } else {
            State::NotLoaded {
                error: "Could not find `ShaderTool.ron` or `ShaderTool.json` in current directory."
                    .to_string(),
            }
        }
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

    fn redraw(&mut self, control_flow: &mut ControlFlow) {
        let mut needs_repaint = self.draw_gui();

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

            match self.state {
                State::Loaded { ref config, .. } | State::ReloadError { ref config, .. } => {
                    // Unwrap because at this point we verified that the current config should run
                    // without problem.
                    needs_repaint |= config.render(&mut target).unwrap();
                    self.egui.paint(&self.display, &mut target);
                    target.finish().unwrap()
                }
                // The config reloaded without error but can still fail to render.
                // So test for an error in the first frame and fall back to the old config if the
                // current config fails to render.
                State::FirstFrame { .. } => {
                    if let State::FirstFrame {
                        old_config,
                        config,
                        kind,
                    } = self.state.take()
                    {
                        match config.render(&mut target).and_then(|x| {
                            self.egui.paint(&self.display, &mut target);
                            target.finish()?;
                            Ok(x)
                        }) {
                            Ok(should_poll) => {
                                self.state = State::Loaded { config, kind };
                                needs_repaint |= should_poll;
                            }
                            Err(e) => {
                                *control_flow = glutin::event_loop::ControlFlow::Poll;
                                if let Some(config) = old_config {
                                    self.state = State::ReloadError {
                                        config,
                                        kind,
                                        error: format!("{:?}", e),
                                    }
                                } else {
                                    self.state = State::NotLoaded {
                                        error: format!("{:?}", e),
                                    }
                                }
                            }
                        }
                    } else {
                        unreachable!();
                    }
                }

                _ => {
                    self.egui.paint(&self.display, &mut target);
                    target.finish().unwrap();
                }
            }
        }

        *control_flow = if !self.should_run {
            glutin::event_loop::ControlFlow::Exit
        } else if needs_repaint {
            self.display.gl_window().window().request_redraw();
            glutin::event_loop::ControlFlow::Poll
        } else {
            glutin::event_loop::ControlFlow::Wait
        };
    }

    pub fn handle_event(&mut self, event: Event<UserEvent>, control_flow: &mut ControlFlow) {
        match event {
            // Platform-dependent event handlers to workaround a winit bug
            // See: https://github.com/rust-windowing/winit/issues/987
            // See: https://github.com/rust-windowing/winit/issues/1619
            Event::RedrawEventsCleared if cfg!(windows) => {
                self.redraw(control_flow);
            }
            Event::RedrawRequested(_) if !cfg!(windows) => {
                self.redraw(control_flow);
            }

            Event::WindowEvent { event, .. } => {
                if !self.egui.on_event(&event) {
                    if let Some(x) = self.state.active_config_mut() {
                        x.handle_window_event(&event)
                    }
                }
                if event == WindowEvent::CloseRequested {
                    *control_flow = glutin::event_loop::ControlFlow::Exit;
                    self.should_run = false;
                    return;
                }

                *control_flow = glutin::event_loop::ControlFlow::Poll;
                self.display.gl_window().window().request_redraw(); // TODO: ask egui if the events warrants a repaint instead
            }
            Event::DeviceEvent { event, .. } => {
                if let Some(x) = self.state.active_config_mut() {
                    x.handle_device_event(&event)
                }
            }
            Event::UserEvent(UserEvent::FileChanged(_)) => {
                match self.state {
                    State::NotLoaded { .. } => {
                        self.state = Self::initial_load_config(&self.display);
                    }
                    State::FirstFrame { .. } => {
                        if let State::FirstFrame {
                            old_config, kind, ..
                        } = self.state.take()
                        {
                            let new_config = match kind {
                                ConfigKind::Ron => Config::load("./ShaderTool.ron", &self.display),
                                ConfigKind::Json => {
                                    Config::load("./ShaderTool.json", &self.display)
                                }
                            };
                            match new_config {
                                Ok(x) => {
                                    self.state = State::FirstFrame {
                                        old_config,
                                        config: Box::new(x),
                                        kind,
                                    }
                                }
                                Err(e) => {
                                    if let Some(config) = old_config {
                                        self.state = State::ReloadError {
                                            config,
                                            kind,
                                            error: format!("{:?}", e),
                                        }
                                    } else {
                                        self.state = State::NotLoaded {
                                            error: format!("{:?}", e),
                                        }
                                    }
                                }
                            }
                        } else {
                            unreachable!()
                        }
                    }
                    State::ReloadError { .. } | State::Loaded { .. } => {
                        if let State::ReloadError { config, kind, .. }
                        | State::Loaded { config, kind } = self.state.take()
                        {
                            let new_config = match kind {
                                ConfigKind::Ron => Config::load("./ShaderTool.ron", &self.display),
                                ConfigKind::Json => {
                                    Config::load("./ShaderTool.json", &self.display)
                                }
                            };
                            match new_config {
                                Ok(x) => {
                                    self.state = State::FirstFrame {
                                        old_config: Some(config),
                                        config: Box::new(x),
                                        kind,
                                    }
                                }
                                Err(e) => {
                                    self.state = State::ReloadError {
                                        config,
                                        kind,
                                        error: format!("{:?}", e),
                                    }
                                }
                            }
                        } else {
                            unreachable!()
                        }
                    }
                }
                *control_flow = glutin::event_loop::ControlFlow::Poll;
                self.display.gl_window().window().request_redraw();
            }
            _ => {}
        }
    }
}
