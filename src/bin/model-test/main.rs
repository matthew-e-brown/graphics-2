use std::error::Error;
use std::process::ExitCode;
use std::sync::mpsc::Receiver;

use glfw::{Context, Glfw, Window, WindowEvent};
use gloog::loader;
use gloog::loader::obj::{ObjModel, ObjVertex};
use gloog_core::types::{
    BufferID,
    BufferTarget,
    BufferUsage,
    ClearMask,
    DrawElementsType,
    DrawMode,
    EnableCap,
    ProgramID,
    ShaderType,
    UniformLocation,
    VertexArrayID,
    VertexAttribType,
};
use gloog_core::GLContext;
use gloog_math::{Mat4, Vec3, Vec4};
use log::info;
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
    let (mut glfw, mut window, events, gl) = init_gl()?;

    gl.clear_color(0.20, 0.20, 0.20, 1.0);
    gl.enable(EnableCap::DepthTest);
    gl.enable(EnableCap::PrimitiveRestartFixedIndex);
    gl.enable(EnableCap::Multisample);

    // Initialize program program
    let program = setup_program(&gl)?;
    gl.use_program(program);

    let uniforms = AllUniforms::get(&gl, program);
    info!("Loaded uniforms: {uniforms:?}");

    let view_matrix = look_at(&Vec3::new(0., 0., 600.), &Vec3::new(0., 0., 0.));
    let proj_matrix = perspective(60.0, 1.00, 0.25, 1000.0);

    gl.uniform_matrix_4fv(uniforms.matrix.proj, false, &[proj_matrix.into()]);
    gl.uniform_matrix_4fv(uniforms.matrix.view, false, &[view_matrix.into()]);

    // Finally load the model
    let model = loader::obj::ObjModel::from_file(model_path, None)?;
    let mut object = Thingy::init(&gl, &model);

    let lights = vec![
        Light::white(Vec3::new(5.0, 5.0, 5.0)),
        /* ... */
    ];

    while !window.should_close() {
        gl.clear(ClearMask::COLOR | ClearMask::DEPTH);

        // Send light uniforms
        gl.uniform_1i(uniforms.num_lights, lights.len() as i32);
        for (i, light) in lights.iter().enumerate() {
            gl.uniform_3fv(uniforms.lights[i].diffuse, &[light.diffuse.into()]);
            gl.uniform_3fv(uniforms.lights[i].ambient, &[light.ambient.into()]);
            gl.uniform_3fv(uniforms.lights[i].specular, &[light.specular.into()]);

            let lp4_ws = Vec4::from3(light.position, 1.0);
            let lp4_vs = view_matrix * lp4_ws;
            let lp3_vs = Vec3::new(lp4_vs[0], lp4_vs[1], lp4_vs[2]);

            gl.uniform_3fv(uniforms.lights[i].position, &[lp3_vs.into()]);
        }

        object.draw(&view_matrix, &uniforms);

        window.swap_buffers();
        glfw.poll_events();

        let time = (glfw.get_time() % f32::MAX as f64) as f32;
        object.rot.x = (time * 20.0).to_radians();
        object.rot.y = (time * 15.0).to_radians();
        object.rot.z = (time * 10.0).to_radians();

        for (_, event) in glfw::flush_messages(&events) {
            if let WindowEvent::Key(glfw::Key::Escape, _, glfw::Action::Press, _) = event {
                window.set_should_close(true);
            }
        }
    }

    Ok(())
}


fn init_gl() -> Result<(Glfw, Window, Receiver<(f64, WindowEvent)>, GLContext), Box<dyn Error>> {
    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS)?;

    glfw.window_hint(glfw::WindowHint::ContextVersion(4, 6));
    glfw.window_hint(glfw::WindowHint::OpenGlProfile(glfw::OpenGlProfileHint::Core));

    glfw.window_hint(glfw::WindowHint::Samples(Some(4)));

    glfw.window_hint(glfw::WindowHint::DoubleBuffer(true));
    glfw.window_hint(glfw::WindowHint::FocusOnShow(true));
    glfw.window_hint(glfw::WindowHint::Focused(true));

    let (mut window, events) = glfw
        .create_window(512, 512, "Model Loading Test", glfw::WindowMode::Windowed)
        .ok_or("could not create the window")?;

    let gl = GLContext::init(|symbol| window.get_proc_address(symbol))?;

    glfw.set_swap_interval(glfw::SwapInterval::Sync(1));
    window.set_resizable(false);
    window.set_key_polling(true);
    window.make_current();

    let (width, height) = window.get_framebuffer_size();
    gl.viewport(0, 0, width, height);

    Ok((glfw, window, events, gl))
}


struct Thingy<'gl, 'a> {
    gl: &'gl GLContext,
    model: &'a ObjModel,
    pub pos: Vec3,
    pub rot: Vec3,
    pub scl: Vec3,
    vao: VertexArrayID,
    _vbo: BufferID,
    _ebo: BufferID,
}

impl<'gl, 'a> Thingy<'gl, 'a> {
    fn init(gl: &'gl GLContext, model: &'a ObjModel) -> Self {
        let vao = gl.create_vertex_array();
        let buffers = gl.create_buffers(2);
        let vbo = buffers[0];
        let ebo = buffers[1];

        gl.bind_vertex_array(vao);

        gl.bind_buffer(BufferTarget::ArrayBuffer, vbo);
        gl.buffer_data(BufferTarget::ArrayBuffer, bytemuck::cast_slice(model.vertex_data()), BufferUsage::StaticDraw);

        gl.vertex_attrib_pointer(0, 3, VertexAttribType::Float, false, ObjVertex::STRIDE, ObjVertex::OFFSET_POSITION);
        gl.vertex_attrib_pointer(1, 3, VertexAttribType::Float, false, ObjVertex::STRIDE, ObjVertex::OFFSET_NORMAL);
        gl.vertex_attrib_pointer(2, 2, VertexAttribType::Float, false, ObjVertex::STRIDE, ObjVertex::OFFSET_TEX_COORD);

        gl.enable_vertex_attrib_array(0);
        gl.enable_vertex_attrib_array(1);
        gl.enable_vertex_attrib_array(2);

        gl.bind_buffer(BufferTarget::ElementArrayBuffer, ebo);
        gl.named_buffer_data(ebo, bytemuck::cast_slice(model.index_data()), BufferUsage::StreamDraw);

        gl.unbind_vertex_array();

        Self {
            gl,
            model,
            pos: Vec3::new(0., 0., 0.),
            rot: Vec3::new(0., 0., 0.),
            scl: Vec3::new(0.5, 0.5, 0.5),
            vao,
            _vbo: vbo,
            _ebo: ebo,
        }
    }

    fn draw(&self, view_matrix: &Mat4, uniforms: &AllUniforms) {
        let &Self { gl, model, vao, .. } = self;

        let model_matrix = model_matrix(&self.pos, &self.rot, &self.scl);
        let normal_matrix = (view_matrix * model_matrix).inverse().transpose();

        gl.uniform_matrix_4fv(uniforms.matrix.model, false, &[model_matrix.into()]);
        gl.uniform_matrix_4fv(uniforms.matrix.normal, false, &[normal_matrix.into()]);

        gl.bind_vertex_array(vao);

        for group in model.groups() {
            let diffuse = group.material.diffuse.unwrap_or(Vec3::new(1., 1., 1.));
            let ambient = group.material.ambient.unwrap_or(Vec3::new(1., 1., 1.));
            let specular = group.material.specular.unwrap_or(Vec3::new(1., 1., 1.));
            let spec_pow = group.material.spec_pow.unwrap_or(30.0);
            let alpha = group.material.alpha.unwrap_or(1.0);

            gl.uniform_3fv(uniforms.material.diffuse, &[diffuse.into()]);
            gl.uniform_3fv(uniforms.material.ambient, &[ambient.into()]);
            gl.uniform_3fv(uniforms.material.specular, &[specular.into()]);
            gl.uniform_1f(uniforms.material.spec_pow, spec_pow);
            gl.uniform_1f(uniforms.material.alpha, alpha);

            let offset = group.indices().start;
            let count = group.indices().count();
            gl.draw_elements(DrawMode::TriangleFan, count, DrawElementsType::UnsignedInt, offset);
        }

        gl.unbind_vertex_array();
    }
}


#[derive(Debug, Clone)]
struct Light {
    pub diffuse: Vec3,
    pub ambient: Vec3,
    pub specular: Vec3,
    pub position: Vec3,
}

impl Light {
    pub fn from_color(color: Vec3, position: Vec3) -> Self {
        Self {
            diffuse: color,
            ambient: color * 0.1,
            specular: Vec3::new(1.0, 1.0, 1.0),
            position,
        }
    }

    pub fn white(position: Vec3) -> Self {
        Self::from_color(Vec3::new(1.0, 1.0, 1.0), position)
    }

    #[allow(unused)]
    pub fn red(position: Vec3) -> Self {
        Self::from_color(Vec3::new(1.0, 0.0, 0.0), position)
    }

    #[allow(unused)]
    pub fn green(position: Vec3) -> Self {
        Self::from_color(Vec3::new(0.0, 1.0, 0.0), position)
    }

    #[allow(unused)]
    pub fn blue(position: Vec3) -> Self {
        Self::from_color(Vec3::new(0.0, 0.0, 1.0), position)
    }
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

fn model_matrix(pos: &Vec3, rot: &Vec3, scl: &Vec3) -> Mat4 {
    let scale = {
        let mut s = Mat4::IDENTITY;
        s[[0, 0]] = scl.x;
        s[[1, 1]] = scl.y;
        s[[2, 2]] = scl.z;
        s
    };

    let translation = {
        let mut t = Mat4::IDENTITY;
        t[[0, 3]] = pos.x;
        t[[1, 3]] = pos.y;
        t[[2, 3]] = pos.z;
        t
    };

    let rotation = {
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
    };

    translation * rotation * scale
}

#[derive(Debug, Clone)]
struct MaterialUniforms {
    diffuse: UniformLocation,
    ambient: UniformLocation,
    specular: UniformLocation,
    spec_pow: UniformLocation,
    alpha: UniformLocation,
}

#[derive(Debug, Clone)]
struct MatrixUniforms {
    proj: UniformLocation,
    view: UniformLocation,
    model: UniformLocation,
    normal: UniformLocation,
}

#[allow(unused)]
#[derive(Debug, Clone)]
struct LightUniforms {
    position: UniformLocation,
    diffuse: UniformLocation,
    ambient: UniformLocation,
    specular: UniformLocation,
}

#[allow(unused)]
#[derive(Debug, Clone)]
struct AllUniforms {
    matrix: MatrixUniforms,
    material: MaterialUniforms,
    num_lights: UniformLocation,
    lights: [LightUniforms; 8],
}

impl AllUniforms {
    pub fn get(gl: &GLContext, program: ProgramID) -> Self {
        Self {
            matrix: MatrixUniforms {
                proj: gl.get_uniform_location(program, "uProjMatrix").unwrap_or(UniformLocation(-1)),
                view: gl.get_uniform_location(program, "uViewMatrix").unwrap_or(UniformLocation(-1)),
                model: gl.get_uniform_location(program, "uModelMatrix").unwrap_or(UniformLocation(-1)),
                normal: gl.get_uniform_location(program, "uNormMatrix").unwrap_or(UniformLocation(-1)),
            },
            material: MaterialUniforms {
                diffuse: gl
                    .get_uniform_location(program, "uMaterial.diffuse")
                    .unwrap_or(UniformLocation(-1)),
                ambient: gl
                    .get_uniform_location(program, "uMaterial.ambient")
                    .unwrap_or(UniformLocation(-1)),
                specular: gl
                    .get_uniform_location(program, "uMaterial.specular")
                    .unwrap_or(UniformLocation(-1)),
                spec_pow: gl
                    .get_uniform_location(program, "uMaterial.specPow")
                    .unwrap_or(UniformLocation(-1)),
                alpha: gl
                    .get_uniform_location(program, "uMaterial.alpha")
                    .unwrap_or(UniformLocation(-1)),
            },
            num_lights: gl.get_uniform_location(program, "uNumLights").unwrap_or(UniformLocation(-1)),
            lights: std::array::from_fn(|i| LightUniforms {
                position: gl
                    .get_uniform_location(program, &format!("uLights[{i}].position"))
                    .unwrap_or(UniformLocation(-1)),
                diffuse: gl
                    .get_uniform_location(program, &format!("uLights[{i}].diffuse"))
                    .unwrap_or(UniformLocation(-1)),
                ambient: gl
                    .get_uniform_location(program, &format!("uLights[{i}].ambient"))
                    .unwrap_or(UniformLocation(-1)),
                specular: gl
                    .get_uniform_location(program, &format!("uLights[{i}].specular"))
                    .unwrap_or(UniformLocation(-1)),
            }),
        }
    }
}
