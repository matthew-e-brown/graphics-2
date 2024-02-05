//! This is a simple example to test the features of Gloog so far[^1]. I suppose this is an integration test.
//!
//! As Gloog and the rest of the code done for this class updates, this example will fall out of date... sort of. I
//! won't remake this example with more advanced features of Gloog (i.e., once I add a materials system); however, if
//! there are any breaking changes to the things currently used, I'll update it (just to get rid of the red squigglies).
//!
//! [^1]: 2023-09-12, commit 9668205

use std::mem::MaybeUninit;

use bytemuck::cast_slice;
use glfw::{Action, Context, Key, OpenGlProfileHint, SwapInterval, WindowEvent, WindowHint, WindowMode};
use gloog_core::types::{
    BufferTarget,
    BufferUsage,
    ClearMask,
    DrawMode,
    EnableCap,
    ProgramID,
    ShaderType,
    VertexAttribType,
};
use gloog_core::GLContext;
use gloog_math::{Matrix4D as Mat4, Vector3D as Vec3};


const POSITION_DATA: [Vec3; 8] = [
    Vec3::new(-0.5, -0.5, 0.5),
    Vec3::new(-0.5, 0.5, 0.5),
    Vec3::new(0.5, 0.5, 0.5),
    Vec3::new(0.5, -0.5, 0.5),
    Vec3::new(-0.5, -0.5, -0.5),
    Vec3::new(-0.5, 0.5, -0.5),
    Vec3::new(0.5, 0.5, -0.5),
    Vec3::new(0.5, -0.5, -0.5),
];

const COLOR_DATA: [Vec3; 8] = [
    Vec3::new(1.0, 0.0, 0.0),
    Vec3::new(0.0, 1.0, 0.0),
    Vec3::new(0.0, 0.0, 1.0),
    Vec3::new(1.0, 1.0, 0.0),
    Vec3::new(0.0, 1.0, 1.0),
    Vec3::new(1.0, 0.0, 1.0),
    Vec3::new(0.5, 0.0, 1.0),
    Vec3::new(0.0, 0.5, 1.0),
];


pub fn main() {
    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();

    glfw.window_hint(WindowHint::ContextVersion(4, 6));
    glfw.window_hint(WindowHint::OpenGlProfile(OpenGlProfileHint::Core));

    glfw.window_hint(WindowHint::DoubleBuffer(true));
    glfw.window_hint(WindowHint::FocusOnShow(true));
    glfw.window_hint(WindowHint::Focused(true));

    let (mut window, events) = glfw
        .create_window(512, 512, "Graphics II - Test 1", WindowMode::Windowed)
        .expect("Could not create the window.");

    let gl = GLContext::init(|s| window.get_proc_address(s)).unwrap();

    glfw.set_swap_interval(SwapInterval::Sync(1));
    window.set_resizable(false);
    window.set_key_polling(true);
    window.make_current();

    let (width, height) = window.get_framebuffer_size();
    gl.viewport(0, 0, width, height);

    gl.clear_color(0.17, 0.17, 0.17, 1.0);
    gl.enable(EnableCap::DepthTest);

    let program = setup_program(&gl);
    gl.use_program(program);

    let vao = gl.create_vertex_array();
    gl.bind_vertex_array(vao);

    let vbo_positions = gl.create_buffer();
    let position_data = cubify(&POSITION_DATA);
    gl.bind_buffer(BufferTarget::ArrayBuffer, vbo_positions);
    gl.buffer_data(BufferTarget::ArrayBuffer, cast_slice(&position_data[..]), BufferUsage::StaticDraw);
    gl.vertex_attrib_pointer(0, 3, VertexAttribType::Float, false, 0, 0);
    gl.enable_vertex_attrib_array(0);

    let vbo_colors = gl.create_buffer();
    let color_data = cubify(&COLOR_DATA);
    gl.bind_buffer(BufferTarget::ArrayBuffer, vbo_colors);
    gl.buffer_data(BufferTarget::ArrayBuffer, cast_slice(&color_data[..]), BufferUsage::StaticDraw);
    gl.vertex_attrib_pointer(1, 3, VertexAttribType::Float, false, 0, 0);
    gl.enable_vertex_attrib_array(1);

    let u_model = gl
        .get_uniform_location(program, "u_model_matrix")
        .expect("couldn't find `u_model_matrix`");
    let u_view = gl
        .get_uniform_location(program, "u_view_matrix")
        .expect("couldn't find `u_view_matrix`");
    let u_proj = gl
        .get_uniform_location(program, "u_proj_matrix")
        .expect("couldn't find `u_proj_matrix`");

    let view_matrix = look_at(&Vec3::new(0.0, 0.0, 2.0), &Vec3::new(0., 0., 0.));
    let proj_matrix = perspective(80.0, 1.00, 0.25, 50.0);

    println!("view: {:#?}", view_matrix);
    println!("proj: {:#?}", proj_matrix);

    let mut cube = Cube::new();

    while !window.should_close() {
        {
            gl.clear(ClearMask::COLOR | ClearMask::DEPTH);

            gl.uniform_matrix_4fv(u_model, false, &[*cube.model_matrix().as_2d_array()]);
            gl.uniform_matrix_4fv(u_view, false, &[*view_matrix.as_2d_array()]);
            gl.uniform_matrix_4fv(u_proj, false, &[*proj_matrix.as_2d_array()]);

            gl.draw_arrays(DrawMode::Triangles, 0, 36);
        }

        {
            let time = (glfw.get_time() % f32::MAX as f64) as f32;
            cube.rotation.x = (time * 20.0).to_radians();
            cube.rotation.y = (time * 15.0).to_radians();
            cube.rotation.z = (time * 10.0).to_radians();
        }

        window.swap_buffers();
        glfw.poll_events();

        for (_, event) in glfw::flush_messages(&events) {
            if let WindowEvent::Key(Key::Escape, _, Action::Press, _) = event {
                window.set_should_close(true);
            }
        }
    }
}


struct Cube {
    pub position: Vec3,
    pub rotation: Vec3,
    pub scale: Vec3,
}

impl Cube {
    pub fn new() -> Self {
        Self {
            position: Vec3::new(0., 0., 0.),
            rotation: Vec3::new(0., 0., 0.),
            scale: Vec3::new(1., 1., 1.),
        }
    }

    pub fn model_matrix(&self) -> Mat4 {
        let scale = {
            let mut s = Mat4::IDENTITY;
            s[[0, 0]] = self.scale.x;
            s[[1, 1]] = self.scale.y;
            s[[2, 2]] = self.scale.z;
            s
        };

        let translation = {
            let mut t = Mat4::IDENTITY;
            t[[0, 3]] = self.position.x;
            t[[1, 3]] = self.position.y;
            t[[2, 3]] = self.position.z;
            t
        };

        let rotation = {
            let sin_x = self.rotation.x.sin();
            let cos_x = self.rotation.x.cos();
            let sin_y = self.rotation.y.sin();
            let cos_y = self.rotation.y.cos();
            let sin_z = self.rotation.z.sin();
            let cos_z = self.rotation.z.cos();

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
        };

        translation * rotation * scale
    }
}


// cspell:words cubify
fn cubify(vertices: &[Vec3; 8]) -> [Vec3; 36] {
    // SAFETY: an array of `MaybeUninit` is always safe to assume initialized, since an uninitialized `MaybeUninit` is
    // technically in a valid, initialized state.
    let mut array: [MaybeUninit<Vec3>; 36] = unsafe { MaybeUninit::uninit().assume_init() };
    let mut i = 0;

    let mut push_quad = |a, b, c, d| {
        let tl = &vertices[a];
        let bl = &vertices[b];
        let br = &vertices[c];
        let tr = &vertices[d];

        // Copy vectors into a new block of six vectors (3 for each of the 2 triangles), and write all six of those into
        // the output array all at once.
        let ptr = array[i].as_mut_ptr() as *mut [Vec3; 6];
        // SAFETY: this memory is currently uninitialized so we can safely write over it; we just got `ptr` from a valid
        // reference so it is guaranteed to be aligned; this array literal is guaranteed by Rust to be packed in memory.
        unsafe { std::ptr::write(ptr, [*tl, *bl, *br, *tl, *br, *tr]) };
        i += 6;
    };

    push_quad(1, 0, 3, 2);
    push_quad(5, 4, 0, 1);
    push_quad(2, 3, 7, 6);
    push_quad(6, 7, 4, 5);
    push_quad(5, 1, 2, 6);
    push_quad(7, 3, 0, 4);

    // SAFETY: all array members have been safely initialized by this point. Mapping over an array like this an calling
    // `assume_init` becomes a bunch of no-ops and gets optimized away properly; I checked with compiler explorer.
    array.map(|maybe| unsafe { maybe.assume_init() })
}


fn setup_program(gl: &GLContext) -> ProgramID {
    let vert = gl.create_shader(ShaderType::Vertex);
    let frag = gl.create_shader(ShaderType::Fragment);

    gl.shader_source(vert, &[&include_str!("./shader-vert.glsl")]);
    gl.shader_source(frag, &[&include_str!("./shader-frag.glsl")]);

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
