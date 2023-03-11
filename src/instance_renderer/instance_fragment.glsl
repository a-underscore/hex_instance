#version 330

#ifdef GL_FRAGMENT_PRECISION_HIGH

precision highp float;

#else

precision mediump float;

#endif

in vec2 tex_pos;
in vec4 tex_color;
in float tex_id;

out vec4 frag_color;

uniform sampler2D tex;

void main(void) {
	frag_color = texture(tex, tex_pos) * tex_color;
}
