use std::sync::{Mutex, OnceLock};

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

use crate::{scale_matrix, trans_matrix};

#[allow(unused)]
const MAX_LIGHTS: u32 = 16;


pub struct Light<'gl> {
    gl: &'gl GLContext,
    id: usize,
    pub ambient: Vec4,
    pub diffuse: Vec4,
    pub specular: Vec4,
    pub position: Vec3,
    pub draw_color: Vec4,
    info: &'static StaticLightInfo,
    uniforms: LightUniforms,
}

struct StaticLightInfo {
    vao: VertexArrayID,
    vertex_count: usize,
    program: ProgramID,
    u_color: UniformLocation,
    u_model_view_matrix: UniformLocation,
    u_projection_matrix: UniformLocation,
}

struct LightUniforms {
    diffuse: UniformLocation,
    ambient: UniformLocation,
    specular: UniformLocation,
    position: UniformLocation,
}


static LIGHT_INFO: OnceLock<StaticLightInfo> = OnceLock::new();
static NEXT_LIGHT_ID: OnceLock<Mutex<usize>> = OnceLock::new();


impl<'gl> Light<'gl> {
    #[allow(unused)]
    pub fn id(&self) -> usize {
        self.id
    }

    pub fn new(
        gl: &'gl GLContext,
        program: ProgramID,
        position: Vec3,
        diffuse: Vec4,
        ambient: Vec4,
        specular: Vec4,
        draw_color_override: Option<Vec4>,
    ) -> Self {
        let info = LIGHT_INFO.get_or_init(|| Self::init(gl));
        let id = Self::next_id();
        let uniforms = Self::get_uniforms(gl, id, program);

        Self {
            gl,
            id,
            ambient,
            diffuse,
            specular,
            position,
            uniforms,
            draw_color: draw_color_override.unwrap_or(diffuse),
            info,
        }
    }


    fn get_uniforms(gl: &GLContext, id: usize, program: ProgramID) -> LightUniforms {
        let diffuse = gl.get_uniform_location(program, &format!("lights[{id}].diffuse")).unwrap_or_default();
        let ambient = gl.get_uniform_location(program, &format!("lights[{id}].ambient")).unwrap_or_default();
        let specular = gl.get_uniform_location(program, &format!("lights[{id}].specular")).unwrap_or_default();
        let position = gl.get_uniform_location(program, &format!("lights[{id}].position")).unwrap_or_default();

        LightUniforms {
            diffuse,
            ambient,
            specular,
            position,
        }
    }


    fn next_id() -> usize {
        let id = NEXT_LIGHT_ID.get_or_init(|| Mutex::new(0));
        let mut id = id.lock().expect("mutex poisoned");
        let out = *id;
        *id += 1;
        out
    }


    fn init(gl: &GLContext) -> StaticLightInfo {
        let vertex_data = {
            let verts = [
                Vec3::new(-0.5, -0.5, 0.5),
                Vec3::new(-0.5, 0.5, 0.5),
                Vec3::new(0.5, 0.5, 0.5),
                Vec3::new(0.5, -0.5, 0.5),
                Vec3::new(-0.5, -0.5, -0.5),
                Vec3::new(-0.5, 0.5, -0.5),
                Vec3::new(0.5, 0.5, -0.5),
                Vec3::new(0.5, -0.5, -0.5),
            ];

            let mut idx = 0;
            let mut buf = [Vec3::default(); 36];

            let mut inc = || {
                let i = idx;
                idx += 1;
                i
            };

            let mut push_quad = |a: usize, b: usize, c: usize, d: usize| {
                let tl = verts[a];
                let bl = verts[b];
                let br = verts[c];
                let tr = verts[d];

                buf[inc()] = tl;
                buf[inc()] = bl;
                buf[inc()] = br;

                buf[inc()] = tl;
                buf[inc()] = br;
                buf[inc()] = tr;
            };

            push_quad(1, 0, 3, 2);
            push_quad(5, 4, 0, 1);
            push_quad(2, 3, 7, 6);
            push_quad(6, 7, 4, 5);
            push_quad(5, 1, 2, 6);
            push_quad(7, 3, 0, 4);

            buf
        };

        const VERT_SRC: &str = include_str!("./shaders/main.vert");
        const FRAG_SRC: &str = include_str!("./shaders/light.frag");
        let program = super::setup_program(gl, VERT_SRC, FRAG_SRC);

        let u_color = gl.get_uniform_location(program, "uColor").unwrap();
        let u_model_view_matrix = gl.get_uniform_location(program, "uModelViewMatrix").unwrap();
        let u_projection_matrix = gl.get_uniform_location(program, "uProjectionMatrix").unwrap();

        let vao = gl.create_vertex_array();
        gl.bind_vertex_array(vao);

        let vbo = gl.create_buffer();
        gl.bind_buffer(BufferTarget::ArrayBuffer, vbo);
        gl.buffer_data(BufferTarget::ArrayBuffer, bytemuck::cast_slice(&vertex_data[..]), BufferUsage::StaticDraw);
        gl.vertex_attrib_pointer(0, 3, VertexAttribType::Float, false, 0, 0);
        gl.enable_vertex_attrib_array(0);

        gl.unbind_vertex_array();

        StaticLightInfo {
            vao,
            vertex_count: vertex_data.len(),
            program,
            u_color,
            u_model_view_matrix,
            u_projection_matrix,
        }
    }

    pub fn set_uniforms(&self, view_matrix: &Mat4) {
        let &Self { gl, ref uniforms, .. } = self;

        gl.uniform(uniforms.diffuse, &self.diffuse);
        gl.uniform(uniforms.ambient, &self.ambient);
        gl.uniform(uniforms.specular, &self.specular);

        let position4 = self.position.to_vec4(1.0);
        let vs_position = (view_matrix * position4).to_vec3();
        gl.uniform(uniforms.position, &vs_position);
    }

    pub fn draw(&self, view_matrix: &Mat4, proj_matrix: &Mat4) {
        let &Self { gl, info, .. } = self;

        gl.use_program(info.program);
        gl.bind_vertex_array(info.vao);

        let model_view = view_matrix * trans_matrix(self.position) * scale_matrix(Vec3::new(0.2, 0.2, 0.2));

        gl.uniform(info.u_color, &self.draw_color);
        gl.uniform(info.u_model_view_matrix, &model_view);
        gl.uniform(info.u_projection_matrix, proj_matrix);

        gl.draw_arrays(DrawMode::Triangles, 0, info.vertex_count);
    }
}
