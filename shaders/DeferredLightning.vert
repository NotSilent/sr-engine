#version 450

layout (location = 0) out vec2 outPosition;

const vec4 ndctl = vec4(-1.0, -1.0, 0.0, 1.0);
const vec4 ndctr = vec4(1.0, -1.0, 0.0, 1.0);
const vec4 ndcbl = vec4(-1.0, 1.0, 0.0, 1.0);
const vec4 ndcbr = vec4(1.0, 1.0, 0.0, 1.0);

const vec4 VERTICES[6] = vec4[6](ndctl, ndcbl, ndctr, ndctr, ndcbl, ndcbr);

void main()
{
    outPosition = VERTICES[gl_VertexIndex].xy / 2.0 + 0.5;
    gl_Position = VERTICES[gl_VertexIndex];

    // Couldn't make this trick work
    // because the triangle has coordinates in [-3, 3] range
    // and I didn't care enough to figure how to pass the positions to sample from
    // gl_Position = vec4(vec2((gl_VertexIndex << 1) & 2, gl_VertexIndex & 2) * 2.0f - 1.0f, 0.0f, 1.0f);
}