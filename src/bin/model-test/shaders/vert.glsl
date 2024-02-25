#version 460 core

layout (location = 0) in vec3 aPosition;
layout (location = 1) in vec3 aNormal;
layout (location = 2) in vec2 aTexCoord;

uniform mat4 uModelMatrix;
uniform mat4 uViewMatrix;
uniform mat4 uProjMatrix;
uniform mat3 uNormMatrix;

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
