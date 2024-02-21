use std::error::Error;
use std::process::ExitCode;
use std::sync::mpsc::Receiver;

use glfw::{Context, Glfw, Window, WindowEvent};
use gloog::{loader, RawModelData};
use gloog_core::types::{BufferUsage, EnableCap, ProgramID, ShaderType};
use gloog_core::GLContext;
use gloog_math::{Mat4, Vec2, Vec3};
use simple_logger::SimpleLogger;


pub fn main() -> ExitCode {
    let Some(model_path) = std::env::args().skip(1).next() else {
        eprintln!("Missing model filepath");
        return ExitCode::FAILURE;
    };

    SimpleLogger::new()
        .with_local_timestamps()
        .with_colors(true)
        .with_level(log::LevelFilter::Debug)
        .env()
        .init()
        .unwrap();

    match run(model_path) {
        Ok(_) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("{e}");
            ExitCode::FAILURE
        },
    }
}


fn run(model_path: String) -> Result<(), Box<dyn Error>> {
    let (glfw, mut window, events, gl) = init_gl()?;

    gl.clear_color(0.20, 0.20, 0.20, 1.0);
    gl.enable(EnableCap::DepthTest);

    // Finally load the model
    let loaded = loader::obj::ObjData::load_from_file(model_path)?;
    let model = loaded.decompose();

    // Initialize program program
    let program = setup_program(&gl)?;

    let vao = gl.create_vertex_array();
    gl.bind_vertex_array(vao);

    // Make buffers for vertices and indices
    let buffers = gl.create_buffers(2);
    let vbo = buffers[0];
    let ebo = buffers[1];

    // hmm... need to actually do some preparation to this data before I can use it in OpenGL. looks like I'll need to
    // go implement those traits and stuff first after all

    // let raw_vertex_data = bytemuck::cast_slice::<Vec3, u8>(model.vertex_data());
    // let raw_tex_coord_data = bytemuck::cast_slice::<Vec2, u8>(model.tex_coord_data());
    // let raw_normal_data = bytemuck::cast_slice::<Vec3, u8>(model.normal_data());

    // gl.named_buffer_data(vbo, raw_vertex_data, BufferUsage::StaticDraw);


    Ok(())
}


fn init_gl() -> Result<(Glfw, Window, Receiver<(f64, WindowEvent)>, GLContext), Box<dyn Error>> {
    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS)?;

    glfw.window_hint(glfw::WindowHint::ContextVersion(4, 6));
    glfw.window_hint(glfw::WindowHint::OpenGlProfile(glfw::OpenGlProfileHint::Core));

    glfw.window_hint(glfw::WindowHint::DoubleBuffer(true));
    glfw.window_hint(glfw::WindowHint::FocusOnShow(true));
    glfw.window_hint(glfw::WindowHint::Focused(true));

    let (mut window, events) = glfw
        .create_window(512, 512, "Model Loading Test", glfw::WindowMode::Windowed)
        .ok_or("could not create the window")?;

    glfw.set_swap_interval(glfw::SwapInterval::Sync(1));
    window.set_resizable(false);
    window.set_key_polling(true);

    let gl = GLContext::init(|symbol| window.get_proc_address(symbol))?;

    window.make_current();

    let (width, height) = window.get_framebuffer_size();
    gl.viewport(0, 0, width, height);

    Ok((glfw, window, events, gl))
}


fn setup_program(gl: &GLContext) -> Result<ProgramID, String> {
    let vert = gl.create_shader(ShaderType::Vertex);
    let frag = gl.create_shader(ShaderType::Fragment);

    gl.shader_source(vert, &[&include_str!("./shaders/vert.glsl")]);
    gl.shader_source(frag, &[&include_str!("./shaders/frag.glsl")]);

    gl.compile_shader(vert)?;
    gl.compile_shader(frag)?;

    let program = gl.create_program();
    gl.attach_shader(program, vert);
    gl.attach_shader(program, frag);

    gl.link_program(program)?;

    gl.detach_shader(program, vert);
    gl.detach_shader(program, frag);
    gl.delete_shader(vert);
    gl.delete_shader(frag);

    Ok(program)
}


fn look_at(from: &Vec3, to: &Vec3) -> Mat4 {
    let world_up = Vec3::UNIT_Y;

    let d = (from - to).norm(); // direction
    let r = world_up.cross(&d).norm(); // right
    let u = d.cross(&r); // up
    let p = -from;

    #[rustfmt::skip]
    return Mat4::new(
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
}

fn perspective(fov_deg: f32, aspect: f32, near_clip: f32, far_clip: f32) -> Mat4 {
    let fov = (fov_deg.to_radians() / 2.0).tan();
    let a = aspect;
    let n = near_clip;
    let f = far_clip;

    #[rustfmt::skip]
    return Mat4::new_cm(
        1.0 / (a * fov),  0.0,          0.0,                        0.0,
        0.0,              1.0 / fov,    0.0,                        0.0,
        0.0,              0.0,         -(f + n) / (f - n),         -1.0,
        0.0,              0.0,         -(2.0 * f * n) / (f - n),    0.0,
    );
}
