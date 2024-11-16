#version 450

layout (push_constant) uniform Push {
    mat4 model;
    mat4 view;
    mat4 projection;
} push;

layout (location = 0) in vec3 inPosition;

void main() {
    gl_Position = push.projection * push.view * push.model * vec4(inPosition, 1.0);
}