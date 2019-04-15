#version 450

layout(input_attachment_index = 0, set = 0, binding = 0) uniform subpassInput u_diffuse;
layout(push_constant) uniform PushConstants {
   vec3 color;
} push_constants;

layout(location = 0) out vec4 f_color;

void main() {
        f_color = vec4(push_constants.color * subpassLoad(u_diffuse).rgb, 1.0);
}
