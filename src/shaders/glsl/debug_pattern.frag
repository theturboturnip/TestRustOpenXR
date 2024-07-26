#version 450
#extension GL_EXT_multiview : require

layout (location=0) in vec2 xy;
layout (location=0) out vec4 FragColor;

void main() {
    FragColor = vec4(xy.x, xy.y, gl_ViewIndex, 1);
}