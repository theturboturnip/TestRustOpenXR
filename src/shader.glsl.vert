#version 330

void main() {
    float x = float(gl_VertexIndex - 1);
    float y = float((gl_VertexIndex & 1) * 2 - 1);
    gl_Position = vec4(x, y, 0, 0);
}