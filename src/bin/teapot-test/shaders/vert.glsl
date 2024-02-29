#version 460 core

layout(location = 0) in vec3 aPosition;
layout(location = 1) in vec3 aNormal;

uniform mat4 uProjectionMatrix = mat4(1.0);
uniform mat4 uModelViewMatrix = mat4(1.0);
uniform mat3 uNormalMatrix = mat3(1.0);

out vec3 vPosition;
out vec3 vNormal;

void main() {
    gl_Position = uProjectionMatrix * uModelViewMatrix * vec4(aPosition, 1.0);
    vPosition = vec3(uModelViewMatrix * vec4(aPosition, 1.0));
    vNormal = uNormalMatrix * aNormal;
}
