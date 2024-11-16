#version 450

layout (push_constant) uniform Push {
    mat4 model;
    mat4 view;
    mat4 projection;
} push;

layout (location = 0) in vec3 inPosition;
layout (location = 1) in vec3 inNormal;

layout (location = 0) out vec3 outNormal;
layout (location = 1) out vec3 outPosition;

void main() {
    mat3 normalMatrix = transpose(inverse(mat3(push.model)));
    outNormal = normalMatrix * inNormal;

    vec4 position = push.model * vec4(inPosition, 1.0);
    outPosition = position.xyz;

    gl_Position = push.projection * push.view * position;
}