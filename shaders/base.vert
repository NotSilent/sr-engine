#version 450

layout (push_constant) uniform PushData
{
    mat4 model;
    mat4 view;
    mat4 projection;
} push_data;

layout (location = 0) in vec3 in_position;

layout (location = 0) out vec3 out_color;

vec2 positions[3] = vec2[](
vec2(0.0, -0.5),
vec2(0.5, 0.5),
vec2(-0.5, 0.5)
);

void main() {
    out_color = vec3(in_position);

    gl_Position = vec4(in_position, 1.0);
}