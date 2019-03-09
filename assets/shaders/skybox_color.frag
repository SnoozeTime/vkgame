#version 450

layout(location = 0) in vec3 frag_tex_coords;
layout(location=0) out vec4 f_color;

layout (push_constant) uniform PushConstants {
        vec4 color;
} push_constants;

void main() {

        f_color = push_constants.color;
}
