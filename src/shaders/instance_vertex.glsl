#version 330

in vec2 position;
in vec2 uv;
in float z;
in mat3 transform;
in vec4 color;

out vec2 tex_pos;
out vec4 tex_color;

uniform mat3 camera_transform;
uniform mat4 camera_view;

void main(void) {
        vec2 pos = (inverse(camera_transform) * transform * vec3(position, 1.0)).xy;

        gl_Position = camera_view * vec4(vec3(pos, z), 1.0);

	tex_pos = uv;
	tex_color = color;
}
