# version 450

layout(location = 0) in vec3 position;
layout(location = 1) in vec2 texcoords;
layout(location = 2) in vec3 normals;

layout(location = 0) out vec3 frag_tex_coords;

layout(binding = 0) uniform Data {
        mat4 model;
        mat4 view;
        mat4 proj;
} uniforms;

void main() {
        frag_tex_coords = position;
        vec4 pos = uniforms.proj * uniforms.view * uniforms.model * vec4(position, 1.0);
        gl_Position = pos.xyzz; 
}
