#version 330 core

uniform sampler2D image;

in vec4 out_color;
in vec2 out_uv;

void main() {
  float d = texture(image, out_uv).r;
  float aaf = fwidth(d);
  float alpha = smoothstep(0.5 - aaf, 0.5 + aaf, d);

  gl_FragColor = vec4(out_color.rgb, alpha);

  //out_color = vec4(1.0, 0.5, 0.2, 1.0);
}

// vim: set et sts=2 sw=2:
