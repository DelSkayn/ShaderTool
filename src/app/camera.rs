use glam::f32::{Quat, Vec2, Vec3};
use glutin::event::{ElementState, MouseButton};

pub trait CameraHandler {
    fn handle_mouse_click(
        &mut self,
        _camera: &mut Camera,
        _btn: MouseButton,
        _state: ElementState,
    ) -> Option<CaptureMouse> {
        None
    }

    fn handle_mouse_move(&mut self, _camera: &mut Camera, _delta: Vec2) -> Option<CaptureMouse> {
        None
    }

    fn handle_mouse_scroll(&mut self, _camera: &mut Camera, _delta: Vec2) {}
}

#[derive(Eq, PartialEq)]
pub enum CaptureMouse {
    Capture,
    Release,
}

#[derive(Clone, Debug, Copy)]
pub struct Camera {
    pub position: Vec3,
    pub rotation: Quat,
    pub fov: f32,
}

impl Camera {
    pub fn new() -> Self {
        Camera {
            position: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            fov: 90.0,
        }
    }
}

pub struct OrbitalCamera {
    currently_pressed: bool,
    distance: f32,
    state: Vec2,
}

impl OrbitalCamera {
    pub fn new() -> OrbitalCamera {
        OrbitalCamera {
            currently_pressed: false,
            distance: 5.0,
            state: Vec2::ZERO,
        }
    }

    fn update_camere(&mut self, camera: &mut Camera) {
        let rotation_y = Quat::from_rotation_y(self.state.x * 0.01);
        let rotation_x = Quat::from_axis_angle(rotation_y * Vec3::X, -self.state.y * 0.01);
        let rotation = (rotation_x * rotation_y).normalize();
        let position = rotation * Vec3::new(0.0, 0.0, -1.0) * self.distance;

        camera.position = position;
        camera.rotation = rotation;
    }
}

impl CameraHandler for OrbitalCamera {
    fn handle_mouse_move(&mut self, camera: &mut Camera, delta: Vec2) -> Option<CaptureMouse> {
        if self.currently_pressed {
            self.state += delta;
        }

        self.update_camere(camera);

        None
    }

    fn handle_mouse_click(
        &mut self,
        _camera: &mut Camera,
        btn: MouseButton,
        state: ElementState,
    ) -> Option<CaptureMouse> {
        if btn == MouseButton::Left && state == ElementState::Released {
            self.currently_pressed = false;
            return Some(CaptureMouse::Release);
        }
        if btn == MouseButton::Left && state == ElementState::Pressed {
            self.currently_pressed = true;
            return Some(CaptureMouse::Capture);
        }

        None
    }

    fn handle_mouse_scroll(&mut self, camera: &mut Camera, delta: Vec2) {
        self.distance += delta.y * 0.004;

        self.update_camere(camera);
    }
}
