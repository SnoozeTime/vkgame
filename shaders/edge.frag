#version 450

layout(location=0) in vec2 uv;
layout(set=0, binding=0) uniform sampler2D diffuseSampler;

layout(location=0) out vec4 f_color;

void main() {
        f_color = texture(diffuseSampler, uv);
}
