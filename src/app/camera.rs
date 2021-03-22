use glam::f32::{Quat, Vec2, Vec3};
use glutin::event::{ElementState, MouseButton, WindowEvent};

pub trait CameraHandler {
    fn handle_event(&mut self, event: WindowEvent, camera: &mut Camera) -> CaptureMouse;
}

pub enum CaptureMouse {
    Capture,
    Release,
}

#[derive(Clone, Debug, Copy)]
pub struct Camera {
    position: Vec3,
    rotation: Quat,
    fov: f32,
}

pub struct OrbitalCamera {
    currently_pressed: bool,
    distance: f32,
    state: Vec2,
    last_position: Vec2,
}

impl OrbitalCamera {
    pub fn new() -> OrbitalCamera {
        OrbitalCamera {
            currently_pressed: false,
            distance: 5.0,
            state: Vec2::ZERO,
            last_position: Vec2::ZERO,
        }
    }
}

impl CameraHandler for OrbitalCamera {
    fn handle_event(&mut self, event: WindowEvent, camera: &mut Camera) -> CaptureMouse {
        match event {
            WindowEvent::MouseInput { state, button, .. } => {
                if button == MouseButton::Left {
                    self.currently_pressed = state == ElementState::Pressed;
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                let pos: [f32; 2] = position.cast().into();
                let pos = Vec2::from(pos);
                let delta = self.last_position - pos;
                if self.currently_pressed {
                    self.state += delta;
                }
            }
            _ => {}
        }

        if self.currently_pressed {
            CaptureMouse::Capture
        } else {
            CaptureMouse::Release
        }
    }
}
