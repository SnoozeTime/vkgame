#version 450

layout(set=0, binding=0) uniform sampler2D diffuseSampler;

layout(location=0) in vec2 uv;
layout(location=0) out vec4 f_color;
void main() {
       f_color = texture(diffuseSampler, uv);
       // float z_n = 2.0 * z_b - 1.0;
       // float z_e = 2.0 * 0.1 * 100. / (100.0 + 0.1 - z_n * (100.0 - 0.1));
       // f_color = vec4(z_e, z_e, z_e, 1.0);
}
