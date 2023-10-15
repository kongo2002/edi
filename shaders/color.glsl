#version 330 core

in vec4 out_color;

layout(location = 0) out vec4 diffuse_color;

void main() {
    diffuse_color = out_color;
}
