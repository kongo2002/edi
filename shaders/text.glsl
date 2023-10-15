#version 330 core

uniform sampler2D image;

in vec4 out_color;
in vec2 out_uv;

layout(location = 0) out vec4 diffuse_color;

void main() {
  float d = texture(image, out_uv).r;
  float aaf = fwidth(d);
  float alpha = smoothstep(0.5 - aaf, 0.5 + aaf, d);

  diffuse_color = vec4(out_color.rgb, alpha);
}

// vim: set et sts=2 sw=2:
