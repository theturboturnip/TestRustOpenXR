#version 450

layout(location=0) out vec2 xy;

void main() {
    vec2 vertices[6] = vec2[6](
        vec2(-1, -1), vec2(1, -1), vec2(1, 1),
        vec2(-1, -1), vec2(1, 1), vec2(-1, 1)
    );
    gl_Position = vec4(vertices[gl_VertexIndex],0,1);
    xy = (vertices[gl_VertexIndex] + 1) / 2;
}