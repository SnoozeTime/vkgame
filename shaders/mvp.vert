#version 450

layout(location = 0) in vec2 position;
layout(location = 1) in vec4 color;

layout(location = 0) out vec4 frag_color;

layout(binding = 0) uniform Data {
    mat4 model;
    mat4 view;
    mat4 proj;
} uniforms;

void main() {
    frag_color = color;
    gl_Position = uniforms.proj * uniforms.view * uniforms.model * vec4(position, 0.0, 1.0);
}

