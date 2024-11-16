#version 450

layout (location = 0) out vec4 outColor;
layout (location = 1) out vec4 outSwapchain;

void main() {
    outColor = vec4(1.0f, 0.0f, 0.0f, 1.0f);
    outSwapchain = vec4(0.0f, 0.0f, 1.0f, 1.0f);
}