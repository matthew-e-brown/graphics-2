#version 460 core

layout (location = 0) in vec3 aPosition;
layout (location = 1) in vec3 aNormal;
layout (location = 2) in vec2 aTexCoord;

uniform mat4 uModelMatrix = mat4(1.0);
uniform mat4 uViewMatrix = mat4(1.0);
uniform mat4 uProjMatrix = mat4(1.0);
uniform mat3 uNormMatrix = mat3(1.0);

out vec3 vPosition;
out vec3 vNormal;
out vec2 vTexCoord;

void main() {
    vec4 aPos4 = vec4(aPosition, 1.0);

    vPosition = vec3(uViewMatrix * uModelMatrix * aPos4);
    vNormal = uNormMatrix * aNormal;
    vTexCoord = aTexCoord;

    gl_Position = uProjMatrix * uViewMatrix * uModelMatrix * aPos4;
}
