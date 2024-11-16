#version 450

layout (location = 0) in vec3 inNormal;
layout (location = 1) in vec3 inPosition;

layout (location = 0) out vec4 outColor;
layout (location = 1) out vec4 outNormal;
layout (location = 2) out vec4 outPosition;

void main() {
    outColor = vec4(1.0, 0.0, 0.0, 1.0);
    outNormal = vec4(inNormal, 0.1); // metallic
    outPosition = vec4(inPosition, 0.3); // roughness
}