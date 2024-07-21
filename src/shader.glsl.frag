#version 450

layout (location=0) in vec2 xy;
layout (location=0) out vec4 FragColor;

void main() {
    FragColor = vec4(xy.x, xy.y, 0, 1);
}