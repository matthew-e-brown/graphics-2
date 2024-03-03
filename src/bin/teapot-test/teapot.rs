use std::sync::OnceLock;

use gloog_core::types::{
    BufferTarget,
    BufferUsage,
    DrawMode,
    ProgramID,
    UniformLocation,
    VertexArrayID,
    VertexAttribType,
};
use gloog_core::GLContext;
use gloog_math::{Mat4, Vec3, Vec4};
use rand::distributions::Uniform;
use rand::Rng;

use crate::light::Light;
use crate::{rotate_matrix, scale_matrix, trans_matrix};


#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct TeapotVertex {
    position: Vec3,
    normal: Vec3,
}

#[derive(Debug, Clone, Copy)]
#[allow(unused)]
pub enum TeapotResolution {
    Low,
    Medium,
    High,
}


pub struct Teapot<'gl> {
    gl: &'gl GLContext,

    pub diffuse: Vec4,
    pub ambient: Vec4,
    pub specular: Vec4,
    pub shininess: f32,

    pub position: Vec3,
    pub rotation: Vec3,
    pub scale: Vec3,

    info: &'static StaticTeapotInfo,
    vao: VertexArrayID,
    vertex_count: usize,

    pub random_seed: [f32; 4],
}


struct StaticTeapotInfo {
    program: ProgramID,

    u_num_lights: UniformLocation,

    u_model_view_matrix: UniformLocation,
    u_projection_matrix: UniformLocation,
    u_normal_matrix: UniformLocation,

    u_diffuse: UniformLocation,
    u_ambient: UniformLocation,
    u_specular: UniformLocation,
    u_shininess: UniformLocation,
}


static STATIC_INFO: OnceLock<StaticTeapotInfo> = OnceLock::new();

static VAO_LOW: OnceLock<VertexArrayID> = OnceLock::new();
static VAO_MEDIUM: OnceLock<VertexArrayID> = OnceLock::new();
static VAO_HIGH: OnceLock<VertexArrayID> = OnceLock::new();

#[derive(Debug, Clone)]
struct ModelData {
    data: &'static [u8],
    num_triangles: usize,
}

impl ModelData {
    pub fn data(&self) -> &'static [u8] {
        &self.data[..]
    }

    pub fn num_triangles(&self) -> usize {
        self.num_triangles
    }

    pub fn num_vertices(&self) -> usize {
        self.num_triangles() * 3
    }
}

const MODEL_LOW: ModelData = ModelData {
    data: include_bytes!("./files/teapot_surface0.norm.bin"),
    num_triangles: 5144,
};

const MODEL_MEDIUM: ModelData = ModelData {
    data: include_bytes!("./files/teapot_surface1.norm.bin"),
    num_triangles: 22885,
};

const MODEL_HIGH: ModelData = ModelData {
    data: include_bytes!("./files/teapot_surface2.norm.bin"),
    num_triangles: 158865,
};


impl<'gl> Teapot<'gl> {
    pub fn new(
        gl: &'gl GLContext,
        resolution: TeapotResolution,
        diffuse: Vec4,
        ambient: Vec4,
        specular: Vec4,
        shininess: f32,
    ) -> Self {
        let info = STATIC_INFO.get_or_init(|| Self::init(gl));
        let (vao, vertex_count) = Self::get_model(gl, resolution);

        Self {
            gl,
            diffuse,
            ambient,
            specular,
            shininess,
            position: Vec3::new(0.0, 0.0, 0.0),
            rotation: Vec3::new(0.0, 0.0, 0.0),
            scale: Vec3::new(1.0, 1.0, 1.0),
            info,
            vao,
            vertex_count,
            random_seed: std::array::from_fn(|_| rand::thread_rng().sample(Uniform::new(0.0, 1.0))),
        }
    }

    fn init(gl: &'gl GLContext) -> StaticTeapotInfo {
        const VERT_SRC: &str = include_str!("./shaders/main.vert");
        const FRAG_SRC: &str = include_str!("./shaders/teapot.frag");
        let program = super::setup_program(gl, VERT_SRC, FRAG_SRC);

        let u_num_lights = gl.get_uniform_location(program, "numLights").unwrap_or_default();

        let u_model_view_matrix = gl.get_uniform_location(program, "uModelViewMatrix").unwrap_or_default();
        let u_projection_matrix = gl.get_uniform_location(program, "uProjectionMatrix").unwrap_or_default();
        let u_normal_matrix = gl.get_uniform_location(program, "uNormalMatrix").unwrap_or_default();

        let u_diffuse = gl.get_uniform_location(program, "material.diffuse").unwrap_or_default();
        let u_ambient = gl.get_uniform_location(program, "material.ambient").unwrap_or_default();
        let u_specular = gl.get_uniform_location(program, "material.specular").unwrap_or_default();
        let u_shininess = gl.get_uniform_location(program, "material.shininess").unwrap_or_default();

        StaticTeapotInfo {
            program,
            u_num_lights,
            u_model_view_matrix,
            u_projection_matrix,
            u_normal_matrix,
            u_diffuse,
            u_ambient,
            u_specular,
            u_shininess,
        }
    }

    fn get_model(gl: &'gl GLContext, resolution: TeapotResolution) -> (VertexArrayID, usize) {
        let (model, vao_lock) = match resolution {
            TeapotResolution::Low => (MODEL_LOW, &VAO_LOW),
            TeapotResolution::Medium => (MODEL_MEDIUM, &VAO_MEDIUM),
            TeapotResolution::High => (MODEL_HIGH, &VAO_HIGH),
        };

        let vao = *vao_lock.get_or_init(|| {
            const F32_BYTES: usize = std::mem::size_of::<f32>();

            let vao = gl.create_vertex_array();
            gl.bind_vertex_array(vao);

            let vbo = gl.create_buffer();
            gl.bind_buffer(BufferTarget::ArrayBuffer, vbo);
            gl.buffer_data(BufferTarget::ArrayBuffer, model.data(), BufferUsage::StaticDraw);
            gl.vertex_attrib_pointer(0, 3, VertexAttribType::Float, false, (6 * F32_BYTES) as isize, 0 * F32_BYTES);
            gl.vertex_attrib_pointer(1, 3, VertexAttribType::Float, false, (6 * F32_BYTES) as isize, 3 * F32_BYTES);
            gl.enable_vertex_attrib_array(0);
            gl.enable_vertex_attrib_array(1);

            gl.unbind_vertex_array();
            vao
        });

        (vao, model.num_vertices())
    }

    pub fn program() -> Option<ProgramID> {
        STATIC_INFO.get().map(|info| info.program)
    }

    pub fn pre_draw(gl: &GLContext, view_matrix: &Mat4, lights: &[Light]) {
        let info = STATIC_INFO.get().unwrap();
        gl.use_program(info.program);
        gl.uniform(info.u_num_lights, &(lights.len() as i32));
        for light in lights {
            light.set_uniforms(view_matrix);
        }
    }

    pub fn draw(&self, view_matrix: &Mat4, proj_matrix: &Mat4) {
        let &Self { gl, info, .. } = self;

        gl.bind_vertex_array(self.vao);

        let ctm = trans_matrix(self.position) * rotate_matrix(self.rotation) * scale_matrix(self.scale);
        let mv_matrix = view_matrix * ctm;
        let norm_matrix = mv_matrix.inverse().transpose().to_mat3();

        gl.uniform(info.u_model_view_matrix, &mv_matrix);
        gl.uniform(info.u_normal_matrix, &norm_matrix);
        gl.uniform(info.u_projection_matrix, proj_matrix);

        gl.uniform(info.u_diffuse, &self.diffuse);
        gl.uniform(info.u_ambient, &self.ambient);
        gl.uniform(info.u_specular, &self.specular);
        gl.uniform(info.u_shininess, &self.shininess);

        gl.draw_arrays(DrawMode::Triangles, 0, self.vertex_count);
    }
}
