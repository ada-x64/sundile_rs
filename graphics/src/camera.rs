use sundile_common::{*, time::Time, input::Input};
use cgmath::*;
use winit::event::*;
use std::f32::consts::FRAC_PI_2;
use wgpu::util::DeviceExt;

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0,
);

#[derive(Debug)]
pub struct Projection {
    aspect: f32,
    fovy: Rad<f32>,
    znear: f32,
    zfar: f32,
}

impl Projection {
    pub fn new<F: Into<Rad<f32>>> (
        width: u32,
        height: u32,
        fovy: F,
        znear: f32,
        zfar: f32,
    ) -> Self {
        Self {
            aspect: width as f32 / height as f32,
            fovy: fovy.into(),
            znear,
            zfar,
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.aspect = width as f32 / height as f32;
    }

    pub fn calc_matrix(&self) -> Matrix4<f32> {
        OPENGL_TO_WGPU_MATRIX * perspective(self.fovy, self.aspect, self.znear, self.zfar)
    }

}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
	pub view_position: [f32; 4],
	pub view_proj: [[f32;4]; 4],
}

impl CameraUniform {
	pub fn new() -> Self {
		Self {
			view_position: [0.0; 4],
			view_proj: cgmath::Matrix4::identity().into(),
		}
	}

	pub fn update_view_proj(&mut self, cam: &Camera, proj: &Projection) {
		self.view_position = cam.pos.to_homogeneous().into();
		self.view_proj = (proj.calc_matrix() * cam.calc_matrix()).into();
	}
}

#[derive(Debug)]
pub struct Camera {
    pub pos: Point3<f32>,
    pitch: Rad<f32>,
    yaw: Rad<f32>,
}

impl Camera {
    pub fn new<
        V: Into<Point3<f32>>,
        Y: Into<Rad<f32>>,
        P: Into<Rad<f32>>,
    >(
        position: V,
        yaw: Y,
        pitch: P,
    ) -> Self {
        Self {
            pos: position.into(),
            yaw: yaw.into(),
            pitch: pitch.into(),
        }
    }

    pub fn calc_matrix(&self) -> Matrix4<f32> {
        Matrix4::look_to_rh(
        self.pos, 
        Vector3::new(
            self.yaw.0.cos(),
            self.pitch.0.sin(),
            self.yaw.0.sin(),
        ).normalize(),
        Vector3::unit_y(),
    )
    }
}

#[derive(Debug)]
pub struct CameraController {
    vel: Vector3<f32>, //floats to accomodate analog sticks (eventually)
    rot: Vector2<Rad<f32>>,
    spd: f32,
    sensitivity: f32,
}

impl CameraController {
    pub fn new(spd: f32, sensitivity: f32) -> Self {
        Self {
            vel: Vector3::new(0.0,0.0,0.0),
            rot: Vector2::new(Rad(0.0), Rad(0.0)),
            spd,
            sensitivity,
        }
    }

    pub fn handle_input(&mut self, input: &Input) {
        // Keys
        self.vel.z = (input.key_held(VirtualKeyCode::Up) || input.key_held(VirtualKeyCode::W)) as u32 as f32
            - ((input.key_held(VirtualKeyCode::Down) || input.key_held(VirtualKeyCode::S)) as u32 as f32);
        self.vel.x = ((input.key_held(VirtualKeyCode::Right) || input.key_held(VirtualKeyCode::D)) as u32 as f32)
            - ((input.key_held(VirtualKeyCode::Left) || input.key_held(VirtualKeyCode::A)) as u32 as f32);
        self.vel.y = input.key_held(VirtualKeyCode::Space) as u32 as f32
            - (input.key_held(VirtualKeyCode::LShift) as u32 as f32);

        // Cursor
        if input.mb_held(MouseButton::Left) {
        let (dx, dy) = input.cursor_diff();
        self.rot.x = Rad(dx as f32);
        self.rot.y = Rad(dy as f32);
        }
    }

    pub fn update(&mut self, cam: &mut Camera, dt: Time) {
        let dt = dt.as_secs() as f32; //this for legacy reasons

        // vel
        let (yaw_sin, yaw_cos) = cam.yaw.sin_cos();
        let forward = Vector3::new(yaw_cos, 0.0, yaw_sin).normalize();
        let right = Vector3::new(-yaw_sin, 0.0, yaw_cos).normalize();
        cam.pos += forward * self.vel.z * self.spd * dt;
        cam.pos += right * self.vel.x * self.spd * dt;
        cam.pos.y += self.vel.y * self.spd * dt;

        // rot
        cam.yaw += self.rot.x * self.sensitivity * dt;
        cam.pitch += -self.rot.y * self.sensitivity * dt;

        self.rot.x = Rad(0.0);
        self.rot.y = Rad(0.0);

        if cam.pitch < Rad(-FRAC_PI_2) {
            cam.pitch = Rad(-FRAC_PI_2)
        }
        if cam.pitch > Rad(FRAC_PI_2) {
            cam.pitch = Rad(FRAC_PI_2)
        }
    }
}

#[derive(Debug)]
pub struct CameraWrapper {
    pub camera: Camera,
	pub uniform: CameraUniform,
	pub buffer: wgpu::Buffer,
	pub bind_group: wgpu::BindGroup,
    pub bind_group_layout: wgpu::BindGroupLayout,
	pub controller: CameraController,
    pub projection: Projection,
}

impl CameraWrapper {
    pub fn new(device: &wgpu::Device, width: u32, height: u32) -> Self {
        let camera = Camera::new(Point3::new(0.0, 0.0, 0.0), Rad(0.0), Rad(0.0));
        let projection = Projection::new(width, height, Rad(std::f32::consts::FRAC_PI_4), 0.01, 1000.0);
        let mut uniform = CameraUniform::new();
        uniform.update_view_proj(&camera, &projection);

        let buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Camera Buffer"),
                contents: bytemuck::cast_slice(&[uniform]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }
        );

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }
            ],
            label: Some("camera_bind_group_layout"),
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: buffer.as_entire_binding(),
                }
            ],
            label: Some("camera_bind_group"),
        });

        let controller = CameraController::new(8.0, 1.0);

        CameraWrapper {
            camera,
            uniform,
            buffer,
            bind_group_layout,
            bind_group,
            controller,
            projection,
        }
    }

    pub fn update(&mut self, dt: Time) {
		self.controller.update(&mut self.camera, dt);
		self.uniform.update_view_proj(&self.camera, &self.projection);
    }

    pub fn handle_input(&mut self, input: &Input) {
        self.controller.handle_input(&input);
    }

    pub fn render(&self, queue: &wgpu::Queue) {
		queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[self.uniform]));
    }
}