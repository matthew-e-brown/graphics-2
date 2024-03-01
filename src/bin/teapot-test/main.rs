mod light;
mod teapot;

use std::f32::consts::PI;
use std::fmt::Debug;
use std::sync::mpsc::Receiver;

use glfw::{Action, Context, Glfw, Key, OpenGlProfileHint, SwapInterval, Window, WindowEvent, WindowHint, WindowMode};
use gloog_core::types::{ClearMask, EnableCap, ProgramID, ShaderType, StringName};
use gloog_core::{GLContext, InitFailureMode};
use gloog_math::{Mat4, Vec3, Vec4};
use light::Light;
use log::{debug, info, log};
use rand::seq::SliceRandom;
use rand::{thread_rng, Rng};
use rand_distr::{StandardNormal, Uniform};

use self::teapot::{Teapot, TeapotResolution};


const NUM_TEAPOTS: usize = 16;
const DEFAULT_RES: TeapotResolution = TeapotResolution::Low;

const LIGHT_COLORS: &[u32] = &[0xFF0000, 0xFFFF00, 0x00FF00, 0x00FFFF, 0x0000FF, 0xFF00FF];

const CAMERA_START_POS: Vec3 = Vec3::new(20.0, 10.0, 20.0);
const CAMERA_START_TGT: Vec3 = Vec3::new(0.0, 0.0, 0.0);
const CAMERA_SPIN_SPEED: f32 = 120.0;
const CAMERA_ZOOM_SPEED: f32 = 20.0;


#[derive(Clone, Copy, Default)]
struct KeyStatus {
    w: bool,
    a: bool,
    s: bool,
    d: bool,
    arrow_up: bool,
    arrow_down: bool,
}

impl KeyStatus {
    pub fn unset_opposites(&mut self) {
        if self.w && self.s {
            self.w = false;
            self.s = false;
        }

        if self.a && self.d {
            self.a = false;
            self.d = false;
        }

        if self.arrow_up && self.arrow_down {
            self.arrow_up = false;
            self.arrow_down = false;
        }
    }

    pub fn any(&self) -> bool {
        self.w || self.a || self.s || self.d || self.arrow_up || self.arrow_down
    }
}

impl IntoIterator for KeyStatus {
    type Item = (Key, bool);
    type IntoIter = std::array::IntoIter<(Key, bool), 6>;

    fn into_iter(self) -> Self::IntoIter {
        [
            (Key::W, self.w),
            (Key::A, self.a),
            (Key::S, self.s),
            (Key::D, self.d),
            (Key::Up, self.arrow_up),
            (Key::Down, self.arrow_down),
        ]
        .into_iter()
    }
}

impl Debug for KeyStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "KeyStatus({} {} {} {} {} {})",
            self.w.then_some('W').unwrap_or('_'),
            self.a.then_some('A').unwrap_or('_'),
            self.s.then_some('S').unwrap_or('_'),
            self.d.then_some('D').unwrap_or('_'),
            self.arrow_up.then_some('↑').unwrap_or('_'),
            self.arrow_down.then_some('↓').unwrap_or('_'),
        ))
    }
}


fn color(hex: u32, k: f32) -> Vec4 {
    let c = hex & 0xFFFFFF;
    let r = ((c >> 16) & 0xFF) as f32;
    let g = ((c >> 8) & 0xFF) as f32;
    let b = ((c >> 0) & 0xFF) as f32;
    Vec4::new(r / 255.0 * k, g / 255.0 * k, b / 255.0 * k, 1.0)
}

fn main() {
    graphics_2::init_logger();

    let teapot_res = std::env::args()
        .skip(1)
        .next()
        .and_then(|arg| match &arg[..] {
            "1" | "low" | "LOW" => Some(TeapotResolution::Low),
            "2" | "med" | "MED" | "medium" | "MEDIUM" => Some(TeapotResolution::Medium),
            "3" | "high" | "HIGH" => Some(TeapotResolution::High),
            _ => None,
        })
        .unwrap_or(DEFAULT_RES);

    let mut rng = thread_rng();

    let (mut glfw, mut window, events, gl) = setup_window();

    let (window_width, window_height) = window.get_framebuffer_size();
    let fw = window_width as f32;
    let fh = window_height as f32;

    gl.clear_color(0.15, 0.15, 0.15, 1.0);
    gl.enable(EnableCap::DepthTest);
    // gl.enable(EnableCap::Multisample);
    gl.enable(EnableCap::DebugOutput);
    gl.enable(EnableCap::CullFace);

    let mut palette = [
        (color(0x049EF4, 1.0), color(0x0B7FFF, 1.0)),
        (color(0x04F49E, 1.0), color(0x0BFF7F, 1.0)),
        (color(0x9E04F4, 1.0), color(0x7F0BFF, 1.0)),
        (color(0x9EF404, 1.0), color(0x7FFF0B, 1.0)),
        (color(0xF4049E, 1.0), color(0xFF0B7F, 1.0)),
        (color(0xF49E04, 1.0), color(0xFF7F0B, 1.0)),
        (color(0xF4F4F4, 1.0), color(0xFF7F0B, 1.0)),
    ];

    palette.shuffle(&mut rng);

    let mut teapots = (0..NUM_TEAPOTS)
        .map(|i| {
            let (base_color, highlight) = palette[i % palette.len()];
            let shininess = rng.sample(Uniform::new(0.0, 1000.0));
            let mut teapot = Teapot::new(&gl, teapot_res, base_color, base_color, highlight, shininess);

            let x = rng.sample(StandardNormal);
            let y = rng.sample(StandardNormal);
            let z = rng.sample(StandardNormal);
            teapot.position = Vec3::new(x, y, z).norm() * 9.0;

            teapot
        })
        .collect::<Vec<_>>();

    let teapot_program = Teapot::program().expect("Teapot has been initialized by now");
    let mut lights = Vec::with_capacity(1 + LIGHT_COLORS.len());

    lights.push(Light::new(
        &gl,
        teapot_program,
        Vec3::new(0.0, 0.0, 0.0),
        color(0xFFFFFF, 0.50),
        color(0x434343, 0.25),
        color(0xDEDEDE, 0.85),
        None,
    ));

    lights.extend(LIGHT_COLORS.iter().enumerate().map(|(i, &light_color)| {
        let angle = (360.0 / (LIGHT_COLORS.len() as f32) * i as f32) / 180.0 * PI;

        let x = angle.cos();
        let y = rng.sample(StandardNormal);
        let z = angle.sin();
        let pos = Vec3::new(x, y, z).norm() * 17.5;

        Light::new(
            &gl,
            teapot_program,
            pos,
            color(light_color, 0.50),
            color(light_color, 0.25),
            color(light_color, 0.90),
            Some(color(light_color, 1.00)), // draw the light itself will full intensity
        )
    }));

    // -----------------------------

    let camera_tgt = CAMERA_START_TGT;
    let mut camera_pos = CAMERA_START_POS;
    let mut key_status = KeyStatus::default();
    let mut teapots_paused = false;

    let mut prev_timestamp = 0.0;

    while !window.should_close() {
        let timestamp = glfw.get_time() as f32;
        let delta_time = timestamp - prev_timestamp;
        prev_timestamp = timestamp;

        glfw.poll_events();

        for (_, event) in glfw::flush_messages(&events) {
            match event {
                WindowEvent::Key(Key::Escape, _, Action::Press, _) => window.set_should_close(true),
                WindowEvent::Key(Key::Space, _, Action::Press, _) => teapots_paused = !teapots_paused,
                WindowEvent::Key(
                    key @ (Key::W | Key::A | Key::S | Key::D | Key::Up | Key::Down),
                    _,
                    action @ (Action::Press | Action::Release),
                    _,
                ) => {
                    match key {
                        Key::W => key_status.w = std::matches!(action, Action::Press),
                        Key::A => key_status.a = std::matches!(action, Action::Press),
                        Key::S => key_status.s = std::matches!(action, Action::Press),
                        Key::D => key_status.d = std::matches!(action, Action::Press),
                        Key::Up => key_status.arrow_up = std::matches!(action, Action::Press),
                        Key::Down => key_status.arrow_down = std::matches!(action, Action::Press),
                        _ => unreachable!(),
                    }

                    debug!("{key_status:?}");
                },
                _ => (),
            }
        }

        handle_input(delta_time, &key_status, &mut camera_pos, &camera_tgt);

        let view_matrix = look_at(&camera_pos, &camera_tgt);
        let proj_matrix = perspective(60.0, fw / fh, 0.1, 100.0);

        gl.clear(ClearMask::COLOR | ClearMask::DEPTH);

        Teapot::pre_draw(&gl, &view_matrix, &lights);
        for (i, teapot) in teapots.iter_mut().enumerate() {
            if !teapots_paused {
                let Teapot { random_seed: rand, .. } = *teapot;
                let amount = rand[3] / 40.0;

                let scale_fn: fn(_) -> _ = |time: f32| (PI * time).sin() * 0.5 + 1.0;

                let scale = scale_fn(timestamp + (i as f32) * 125.0 * amount);
                teapot.scale = Vec3::new(scale, scale, scale);

                let angle = Vec3::new(rand[0], rand[1], rand[2]);
                teapot.rotation += angle * amount;
            }

            teapot.draw(&view_matrix, &proj_matrix);
        }

        for light in &lights {
            light.draw(&view_matrix, &proj_matrix);
        }

        window.swap_buffers();
    }
}


fn setup_window() -> (Glfw, Window, Receiver<(f64, WindowEvent)>, GLContext) {
    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();

    glfw.window_hint(WindowHint::ContextVersion(4, 6));
    glfw.window_hint(WindowHint::OpenGlProfile(OpenGlProfileHint::Core));

    glfw.window_hint(WindowHint::DoubleBuffer(true));
    glfw.window_hint(WindowHint::FocusOnShow(true));
    glfw.window_hint(WindowHint::Focused(true));

    // glfw.window_hint(WindowHint::Samples(Some(4)));

    let (mut window, events) = glfw
        .create_window(1200, 900, "Graphics II - Teapot Test", WindowMode::Windowed)
        .expect("Could not create the window.");

    let mut gl = GLContext::init(|s| window.get_proc_address(s), InitFailureMode::WarnAndContinue).unwrap();

    gl.debug_message_callback(move |message| {
        let lvl = message.severity.log_level();
        let str = message.body;
        log!(lvl, "{str}");
    });

    glfw.set_swap_interval(SwapInterval::Sync(1));
    window.set_resizable(false);
    window.set_key_polling(true);
    window.make_current();

    let (width, height) = window.get_framebuffer_size();
    gl.viewport(0, 0, width, height);

    let ver = gl.get_string(StringName::Version);
    let vnd = gl.get_string(StringName::Vendor);
    info!("loaded OpenGL version {ver} with vendor {vnd}");

    (glfw, window, events, gl)
}


fn setup_program(gl: &GLContext, vert_src: &str, frag_src: &str) -> ProgramID {
    let vert = gl.create_shader(ShaderType::Vertex);
    let frag = gl.create_shader(ShaderType::Fragment);

    gl.shader_source(vert, [vert_src]);
    gl.shader_source(frag, [frag_src]);

    gl.compile_shader(vert).unwrap();
    gl.compile_shader(frag).unwrap();

    let program = gl.create_program();
    gl.attach_shader(program, vert);
    gl.attach_shader(program, frag);

    gl.link_program(program).unwrap();

    gl.detach_shader(program, vert);
    gl.detach_shader(program, frag);
    gl.delete_shader(vert);
    gl.delete_shader(frag);

    program
}

fn handle_input(delta_time: f32, keys: &KeyStatus, camera_pos: &mut Vec3, camera_tgt: &Vec3) {
    let mut keys = *keys;

    keys.unset_opposites();
    if !keys.any() {
        return;
    }

    let cam_spin_speed = if (keys.w || keys.s) && (keys.a || keys.d) {
        CAMERA_SPIN_SPEED * (PI / 4.0).cos()
    } else {
        CAMERA_SPIN_SPEED
    };

    for key in keys.into_iter().filter_map(|(key, pressed)| pressed.then_some(key)) {
        let target_to_cam = *camera_pos - camera_tgt;
        let target_dist = target_to_cam.mag();

        let cam_forward = target_to_cam.norm();
        let cam_right = Vec3::new(0.0, 1.0, 0.0).cross(&cam_forward);
        let cam_up = cam_forward.cross(&cam_right);

        if let Key::W | Key::A | Key::S | Key::D = key {
            let spin_speed = cam_spin_speed * delta_time;
            let (axis, spin_speed) = match key {
                Key::W => (cam_right, -spin_speed),
                Key::S => (cam_right, spin_speed),
                Key::A => (cam_up, -spin_speed),
                Key::D => (cam_up, spin_speed),
                _ => unreachable!(),
            };

            let r_mat = axis_angle_matrix(spin_speed, axis);
            let r_cam = r_mat * Vec4::from3(*camera_pos, 1.0);
            *camera_pos = Vec3::new(r_cam.x, r_cam.y, r_cam.z);
        } else if let Key::Up | Key::Down = key {
            let mut zoom_speed = CAMERA_ZOOM_SPEED * delta_time;

            if let Key::Up = key {
                zoom_speed *= -1.0;
            }

            let new_distance = target_dist + zoom_speed;
            let clamped_dist = new_distance.max(1.0).min(50.0);

            *camera_pos = clamped_dist * cam_forward;
        }
    }
}


fn scale_matrix(scl: Vec3) -> Mat4 {
    let mut s = Mat4::IDENTITY;
    s[[0, 0]] = scl.x;
    s[[1, 1]] = scl.y;
    s[[2, 2]] = scl.z;
    s
}

fn trans_matrix(pos: Vec3) -> Mat4 {
    let mut t = Mat4::IDENTITY;
    t[[0, 3]] = pos.x;
    t[[1, 3]] = pos.y;
    t[[2, 3]] = pos.z;
    t
}

fn rotate_matrix(rot: Vec3) -> Mat4 {
    let sin_x = rot.x.sin();
    let cos_x = rot.x.cos();
    let sin_y = rot.y.sin();
    let cos_y = rot.y.cos();
    let sin_z = rot.z.sin();
    let cos_z = rot.z.cos();

    #[rustfmt::skip]
    let x = Mat4::new(
        1.0,        0.0,        0.0,    0.0,
        0.0,        cos_x,     -sin_x,  0.0,
        0.0,        sin_x,      cos_x,  0.0,
        0.0,        0.0,        0.0,    1.0,
    );

    #[rustfmt::skip]
    let y = Mat4::new(
        cos_y,      0.0,        sin_y,  0.0,
        0.0,        1.0,        0.0,    0.0,
       -sin_y,      0.0,        cos_y,  0.0,
        0.0,        0.0,        0.0,    1.0,
    );

    #[rustfmt::skip]
    let z = Mat4::new(
        cos_z,     -sin_z,      0.0,    0.0,
        sin_z,      cos_z,      0.0,    0.0,
        0.0,        0.0,        1.0,    0.0,
        0.0,        0.0,        0.0,    1.0,
    );

    x * y * z
}

fn axis_angle_matrix(angle: f32, axis: Vec3) -> Mat4 {
    let Vec3 { x, y, z } = axis.norm();

    let c = angle.to_radians().cos();
    let omc = 1.0 - c;
    let s = angle.to_radians().sin();

    #[rustfmt::skip]
    let m = Mat4::new(
          c + x*x*omc, x*y*omc - z*s, x*z*omc + y*s, 0.0,
        y*x*omc + z*s,   c + y*y*omc, y*z*omc - x*s, 0.0,
        z*x*omc - y*s, z*y*omc + x*s,   c + z*z*omc, 0.0,
                  0.0,           0.0,           0.0, 1.0,
    );

    m
}

fn look_at(from: &Vec3, to: &Vec3) -> Mat4 {
    let world_up = Vec3::UNIT_Y;

    let d = (from - to).norm(); // direction
    let r = world_up.cross(&d).norm(); // right
    let u = d.cross(&r); // up
    let p = -from;

    #[rustfmt::skip]
    let m = Mat4::new(
        r.x, r.y, r.z, 0.0,
        u.x, u.y, u.z, 0.0,
        d.x, d.y, d.z, 0.0,
        0.0, 0.0, 0.0, 1.0,
    ) * Mat4::new(
        1.0, 0.0, 0.0, p.x,
        0.0, 1.0, 0.0, p.y,
        0.0, 0.0, 1.0, p.z,
        0.0, 0.0, 0.0, 1.0,
    );
    m
}

fn perspective(fov_deg: f32, aspect: f32, near_clip: f32, far_clip: f32) -> Mat4 {
    let fov = (fov_deg.to_radians() / 2.0).tan();
    let a = aspect;
    let n = near_clip;
    let f = far_clip;

    #[rustfmt::skip]
    let m = Mat4::new_cm(
        1.0 / (a * fov),  0.0,          0.0,                        0.0,
        0.0,              1.0 / fov,    0.0,                        0.0,
        0.0,              0.0,         -(f + n) / (f - n),         -1.0,
        0.0,              0.0,         -(2.0 * f * n) / (f - n),    0.0,
    );
    m
}
