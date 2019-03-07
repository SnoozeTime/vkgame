# version 450

layout(location = 0) in vec3 position;
layout(location = 1) in vec2 texcoords;
layout(location = 2) in vec3 normals;

layout(binding = 0) uniform Data {
        mat4 model;
        mat4 view;
        mat4 proj;
} uniforms;

void main() {
        vec4 pos = vec4(position, 1.0);
        vec4 scaled_pos = pos + normalize(vec4(normals, 1.0)) * 0.2;
        gl_Position = uniforms.proj * uniforms.view * uniforms.model * scaled_pos;
}
