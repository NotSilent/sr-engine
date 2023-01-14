#version 450

layout (push_constant) uniform PushData
{
    mat4 model;
    mat4 view;
    mat4 projection;
} push_data;

layout (location = 0) in vec3 in_position;

layout (location = 0) out vec3 out_color;

void main() {
    out_color = vec3(1.0, 0.0, 0.0);

    gl_Position = push_data.projection * push_data.view * push_data.model * vec4(in_position, 1.0);
}