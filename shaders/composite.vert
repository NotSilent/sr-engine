#version 450

layout(location = 0) out vec2 outPos;

void main()
{
    outPos = vec2((gl_VertexIndex << 1) & 2, gl_VertexIndex & 2) * 2.0f - 1.0f;
    gl_Position = vec4(outPos, 0.0f, 1.0f);
}