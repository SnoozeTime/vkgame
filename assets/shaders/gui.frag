#version 450

layout (location = 0) in vec2 inUv;
layout (location = 1) in vec4 inColor;
layout (binding = 0) uniform sampler2D fontSampler;

layout(location = 0) out vec4 f_color;

void main() {
        f_color = inColor * texture(fontSampler, inUv);
}
