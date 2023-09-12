#version 460 core

layout (location = 0) in vec3 a_position;
layout (location = 1) in vec3 a_color;

uniform mat4 u_model_matrix;
uniform mat4 u_view_matrix;
uniform mat4 u_proj_matrix;

out vec3 vert_color;

void main() {
    // OpenGL matrices, like ours, are column-major; but, `*` operators are not overloaded, so `Mv`
    // for multiplying 4x4 * 4x1 is written `vM`.
    gl_Position = u_proj_matrix * u_view_matrix * u_model_matrix * vec4(a_position, 1.0);
    vert_color = a_color;
}
