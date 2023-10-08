#version 330 core

uniform vec2 resolution;

layout (location = 0) in vec2 pos;
layout (location = 1) in vec4 color;
layout (location = 2) in vec2 uv;

out vec4 out_color;
out vec2 out_uv;

vec2 camera_project(vec2 point)
{
    return 2.0 * (point - vec2(0.0, 0.0)) * 1.0 / resolution;
}

void main() {
  gl_Position = vec4(camera_project(pos), 0, 1.0);

  out_color = color;
  out_uv = uv;
}
