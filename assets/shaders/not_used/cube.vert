#version 450

layout(location = 0) in vec3 position;
layout(location = 1) in vec2 texcoords;


layout(location = 0) out vec4 frag_color;
layout(location = 1) out vec2 frag_tex_coords;

layout(binding = 0) uniform Data {
    mat4 model;
    mat4 view;
    mat4 proj;
} uniforms;

void main() {
    frag_color = vec4(position, 1.0);
    frag_tex_coords = texcoords;
    gl_Position = uniforms.proj * uniforms.view * uniforms.model * vec4(position, 1.0);
}

