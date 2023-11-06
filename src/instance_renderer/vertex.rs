vulkano_shaders::shader! {
    ty: "vertex",
    src: r"
#version 450

layout(location = 0) in vec2 position;
layout(location = 1) in vec2 uv;
layout(location = 2) in float z;
layout(location = 3) in vec4 color;
layout(location = 4) in mat3 transform;


layout(location = 0) out vec2 tex_pos;
layout(location = 1) out vec4 tex_color;

layout(set = 0, binding = 0) uniform View {
    mat3 camera_transform;
    mat4 camera_proj;
};

void main(void) {
        vec2 pos = (vec3(position, 1.0) * transform * inverse(camera_transform)).xy;

        gl_Position = vec4(vec3(pos, z), 1.0) * camera_proj;

    	tex_pos = uv;
        tex_color = color;
}
        ",
}
