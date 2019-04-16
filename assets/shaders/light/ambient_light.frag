#version 450

layout(set = 0, binding = 0) uniform sampler2D u_diffuse;
layout(push_constant) uniform PushConstants {
   vec3 color;
} push_constants;

layout(location = 1) in vec2 uv;
layout(location = 0) out vec4 f_color;

void main() {
        f_color = vec4(push_constants.color * texture(u_diffuse, uv).rgb, 1.0);
}
