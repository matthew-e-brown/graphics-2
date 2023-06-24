use std::mem::size_of;

use gfx::gl;
use gfx::gl::types::*;
use gfx::glfw::{self, Context, WindowMode, WindowEvent, Key};
use math::Vec3;


const VERT_BUFFER_SIZE: usize = size_of::<[[Vec3; 2]; 3]>();
const VERTICES: [[Vec3; 2]; 3] = [
    // Position, then color
    [Vec3::new(-0.5, -0.5, 0.0), Vec3::new(1.0, 0.0, 0.0)],
    [Vec3::new(0.5, -0.5, 0.0), Vec3::new(0.0, 1.0, 0.0)],
    [Vec3::new(0.0, 0.5, 0.0), Vec3::new(0.0, 0.0, 1.0)],
];

// Just include the entire source-code of the shaders in the binary, for now
const VERT_SHADER_STR: &str = include_str!("./shader-vert.glsl");
const FRAG_SHADER_STR: &str = include_str!("./shader-frag.glsl");


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

    let _vbo = unsafe {
        // Create our VBO on the graphics card
        let mut vbo = 0;
        gl::CreateBuffers(1, &mut vbo);

        // Send our data
        let size_of = VERT_BUFFER_SIZE
            .try_into()
            .expect("Vertex data is too large to fit inside `isize`.");
        let vert_ptr = VERTICES.as_ptr().cast();
        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
        gl::BufferData(gl::ARRAY_BUFFER, size_of, vert_ptr, gl::STATIC_DRAW);

        vbo
    };

    let program = unsafe { create_program(VERT_SHADER_STR, FRAG_SHADER_STR) }.unwrap();

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

            gl::UseProgram(program);
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
                }
                _ => (),
            }
        }
    }
}


/// Compiles a single shader from source.
unsafe fn compile_shader(shader_type: GLuint, source: &str) -> Result<GLuint, String> {
    let shader = gl::CreateShader(shader_type);
    let src_ptr = source.as_bytes().as_ptr().cast();
    let src_len = source.len().try_into().map_err(|_| "Shader source is too long.".to_owned())?;

    // glShaderSource expects two *arrays*, but since it expects C-style arrays and we aren't passing multiple sets of
    // buffers, we can just pass the pointers.
    gl::ShaderSource(shader, 1, &src_ptr, &src_len);
    gl::CompileShader(shader);

    // Ask OpenGL if it was successful
    let mut success = 0;
    gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut success);

    if (success as GLboolean) == gl::FALSE {
        let mut log_size = 0;
        gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut log_size);

        // The null pointer is for the out parameter for how long the string it spits out is. We have pre-checked this,
        // so we don't really care.
        let mut buffer = vec![0; log_size as usize];
        gl::GetShaderInfoLog(shader, log_size, std::ptr::null_mut(), buffer.as_mut_ptr().cast());

        // `lossy` because that way we won't crash if the log had invalid unicode; just get a `?` character.
        let log_output = String::from_utf8_lossy(&buffer[..]);

        gl::DeleteShader(shader);

        Err(log_output.into_owned())
    } else {
        Ok(shader)
    }
}


unsafe fn create_program(vert_src: &str, frag_src: &str) -> Result<GLuint, String> {
    // Compile both shaders, failing out of this one if either shader fails
    let vert_shader = compile_shader(gl::VERTEX_SHADER, vert_src.trim())?;
    let frag_shader = compile_shader(gl::FRAGMENT_SHADER, frag_src.trim())?;

    // Create our program and link the shaders
    let program = gl::CreateProgram();
    gl::AttachShader(program, vert_shader);
    gl::AttachShader(program, frag_shader);
    gl::LinkProgram(program);

    // Check if we were successful
    let mut success = 0;
    gl::GetProgramiv(program, gl::LINK_STATUS, &mut success);

    if (success as GLboolean) == gl::FALSE {
        let mut log_size = 0;
        gl::GetProgramiv(program, gl::INFO_LOG_LENGTH, &mut log_size);

        let mut buffer = vec![0; log_size as usize];
        gl::GetProgramInfoLog(program, log_size, std::ptr::null_mut(), buffer.as_mut_ptr().cast());

        gl::DeleteProgram(program);
        gl::DeleteShader(vert_shader);
        gl::DeleteShader(frag_shader);

        let log_output = String::from_utf8_lossy(&buffer);
        Err(log_output.into_owned())
    } else {
        // In the future, we will save these shaders so that we don't have to recompile; for now, we only need the
        // linked program.
        gl::DeleteShader(vert_shader);
        gl::DeleteShader(frag_shader);

        Ok(program)
    }
}
