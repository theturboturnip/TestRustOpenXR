#version 450
#extension GL_EXT_multiview : require

layout(set=0, binding=0, std140) uniform Eyes {
    mat4 eye_screen_from_world[2];
};

layout(set=0, binding=1, std140) uniform PerObject {
    mat4 world_from_model;
};

layout(location=0) out vec2 xy;

void main() {
    vec2 vertices[6] = vec2[6](
        vec2(-1, -1), vec2(1, -1), vec2(1, 1),
        vec2(-1, -1), vec2(1, 1), vec2(-1, 1)
    );
    gl_Position = eye_screen_from_world[gl_ViewIndex] * world_from_model * vec4(vertices[gl_VertexIndex] * 0.5,-2,1);
    xy = (vertices[gl_VertexIndex] + 1) / 2;
}