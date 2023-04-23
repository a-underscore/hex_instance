#version 330

in vec2 tex_pos;
in vec4 tex_color;

out vec4 frag_color;

uniform sampler2D tex;

void main(void) {
	frag_color = texture(tex, tex_pos) * tex_color;
}
