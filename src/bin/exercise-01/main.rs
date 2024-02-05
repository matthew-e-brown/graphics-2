use std::mem::size_of;

use bytemuck::cast_slice;
use glfw::{self, Context, Key, WindowEvent, WindowMode};
use gloog_core::types::{BufferTarget, BufferUsage, ClearMask, DrawMode, ProgramID, ShaderType, VertexAttribType};
use gloog_core::GLContext;
use gloog_math::Vector3D as Vec3;


const VERTICES: [[Vec3; 2]; 3] = [
    // Position, then color
    [Vec3::new(-0.5, -0.5, 0.0), Vec3::new(1.0, 0.0, 0.0)],
    [Vec3::new(0.5, -0.5, 0.0), Vec3::new(0.0, 1.0, 0.0)],
    [Vec3::new(0.0, 0.5, 0.0), Vec3::new(0.0, 0.0, 1.0)],
];


pub fn main() {
    // GLFW set up
    // -----------------------------------------------------------------

    // Create main GLFW context
    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();

    // Request OpenGL Core 4.6
    glfw.window_hint(glfw::WindowHint::ContextVersion(4, 6));
    glfw.window_hint(glfw::WindowHint::OpenGlProfile(glfw::OpenGlProfileHint::Core));

    // Other window settings
    glfw.window_hint(glfw::WindowHint::DoubleBuffer(true)); // use double buffering
    glfw.window_hint(glfw::WindowHint::FocusOnShow(true)); // focus the window when it is shown
    glfw.window_hint(glfw::WindowHint::Focused(true)); // focus the window on creation

    let (mut window, events) = glfw
        .create_window(512, 512, "Graphics II - Exercise 1 - Hello World!", WindowMode::Windowed)
        .expect("Could not create the window.");

    // Pass all calls to load OpenGL symbols to GLFW
    let gl = GLContext::init(|s| window.get_proc_address(s)).unwrap();

    glfw.set_swap_interval(glfw::SwapInterval::Sync(1));
    window.set_resizable(false);
    window.set_key_polling(true);
    window.make_current();

    // Now that the window is loaded, initialize the viewport to the same size as the framebuffer
    let (width, height) = window.get_framebuffer_size();
    gl.viewport(0, 0, width, height);

    // OpenGL rendering set up
    // -----------------------------------------------------------------

    let program = compile_and_link_program(&gl).unwrap();
    gl.use_program(program);

    let vbo = gl.create_buffer();
    gl.bind_buffer(BufferTarget::ArrayBuffer, vbo);
    gl.buffer_data(BufferTarget::ArrayBuffer, cast_slice(&VERTICES[..]), BufferUsage::StaticDraw);

    let vao = gl.create_vertex_array();
    gl.bind_vertex_array(vao);

    let vec_size = size_of::<Vec3>();
    let stride = vec_size as isize * 2;

    gl.vertex_attrib_pointer(0, 3, VertexAttribType::Float, false, stride, vec_size * 0);
    gl.vertex_attrib_pointer(1, 3, VertexAttribType::Float, false, stride, vec_size * 1);
    gl.enable_vertex_attrib_array(0);
    gl.enable_vertex_attrib_array(1);

    // Draw loop
    // -----------------------------------------------------------------

    while !window.should_close() {
        gl.clear_color(0.17, 0.17, 0.17, 1.0);
        gl.clear(ClearMask::COLOR);
        gl.bind_vertex_array(vao);
        gl.draw_arrays(DrawMode::Triangles, 0, VERTICES.len());

        window.swap_buffers();
        glfw.poll_events();

        for (_, event) in glfw::flush_messages(&events) {
            match event {
                // Do nothing except close when ESC is pressed
                WindowEvent::Key(Key::Escape, _, glfw::Action::Press, _) => {
                    window.set_should_close(true);
                },
                _ => (),
            }
        }
    }
}


fn compile_and_link_program(gl: &GLContext) -> Result<ProgramID, String> {
    let vert_shader = gl.create_shader(ShaderType::Vertex);
    let frag_shader = gl.create_shader(ShaderType::Fragment);

    // Just include the entire source-code of the shaders in the binary, for now
    gl.shader_source(vert_shader, &[include_str!("./shader-vert.glsl")]);
    gl.shader_source(frag_shader, &[include_str!("./shader-frag.glsl")]);

    gl.compile_shader(vert_shader)?;
    gl.compile_shader(frag_shader)?;

    let program = gl.create_program();
    gl.attach_shader(program, vert_shader);
    gl.attach_shader(program, frag_shader);

    gl.link_program(program)?;

    gl.detach_shader(program, vert_shader);
    gl.detach_shader(program, frag_shader);

    Ok(program)
}
