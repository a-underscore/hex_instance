#version 330

in vec2 position;
in vec2 uv;
in float z;
in mat3 transform;
in vec4 color;

out vec2 tex_pos;
out vec4 tex_color;
out float tex_id;

uniform mat3 camera_transform;
uniform mat4 camera_view;

void main(void) {
        vec2 pos = (vec3(position, 1.0) * transform * inverse(camera_transform)).xy;

        gl_Position = vec4(vec3(pos, z), 1.0) * camera_view;

	tex_pos = uv;
	tex_color = color;
	tex_id = id;
}
