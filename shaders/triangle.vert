#version 450

layout(location = 0) in vec2 position;
layout(location = 1) in vec4 color;

layout(location = 0) out vec4 frag_color;

layout(set = 0, binding = 0) uniform Data {
    vec3 offset;
} uniforms;

void main() {
    frag_color = color;
    gl_Position = vec4(uniforms.offset, 1.0) + vec4(position, 0.0, 1.0);
}

