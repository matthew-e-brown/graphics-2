use std::mem::size_of;

use glfw::{self, Context, Key, WindowEvent, WindowMode};
use gloog::{bindings as gl, Buffer, BufferTarget, BufferUsage, Program, Shader, ShaderType};
use gloog_math::Vec3;


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
    gl::load_with(|s| window.get_proc_address(s));

    glfw.set_swap_interval(glfw::SwapInterval::Sync(1));
    window.set_resizable(false);
    window.set_key_polling(true);
    window.make_current();

    // Now that the window is loaded, initialize the viewport to the same size as the framebuffer
    unsafe {
        let (width, height) = window.get_framebuffer_size();
        gl::Viewport(0, 0, width, height);
    }

    // OpenGL rendering set up
    // -----------------------------------------------------------------

    let program = compile_and_link_program().unwrap();
    program.use_program();

    let mut vbo = Buffer::create();
    let mut vbo_bound = vbo.bind_mut(BufferTarget::ArrayBuffer);
    vbo_bound.data(&VERTICES, BufferUsage::StaticDraw);

    let vao = unsafe {
        let mut vao = 0;
        gl::CreateVertexArrays(1, &mut vao);
        gl::BindVertexArray(vao);

        let stride = size_of::<Vec3>() as i32 * 2;
        let f_size = size_of::<f32>() as i32;

        gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, stride, (f_size * 0) as *const _);
        gl::VertexAttribPointer(1, 3, gl::FLOAT, gl::FALSE, stride, (f_size * 3) as *const _);
        gl::EnableVertexAttribArray(0);
        gl::EnableVertexAttribArray(1);

        vao
    };

    // Draw loop
    // -----------------------------------------------------------------

    while !window.should_close() {
        unsafe {
            gl::ClearColor(0.17, 0.17, 0.17, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
            gl::BindVertexArray(vao);
            gl::DrawArrays(gl::TRIANGLES, 0, VERTICES.len() as i32);
        }

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


fn compile_and_link_program() -> Result<Program, String> {
    let mut vert_shader = Shader::new(ShaderType::Vertex).map_err(|e| e.to_string())?;
    let mut frag_shader = Shader::new(ShaderType::Fragment).map_err(|e| e.to_string())?;

    // Just include the entire source-code of the shaders in the binary, for now
    vert_shader.set_source(&[include_str!("./shader-vert.glsl")]);
    frag_shader.set_source(&[include_str!("./shader-frag.glsl")]);

    vert_shader.compile()?;
    frag_shader.compile()?;

    let mut program = Program::new().map_err(|e| e.to_string())?;
    program.attach_shader(&vert_shader);
    program.attach_shader(&frag_shader);

    program.link()?;

    program.detach_shader(&vert_shader);
    program.detach_shader(&frag_shader);

    Ok(program)
}
