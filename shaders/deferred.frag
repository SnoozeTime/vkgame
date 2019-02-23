#version 450

layout(location = 0) out vec4 f_color;
layout(location = 1) out vec3 f_normal;
layout(location = 0) in vec4 frag_color;
layout(location = 1) in vec2 frag_tex_coords;
layout(location = 2) in vec3 frag_position;
layout(location = 3) in vec3 frag_normal;

layout(set = 1, binding = 0) uniform sampler2D texSampler;
layout(set = 1, binding = 1) uniform Data {
        vec3 color;
        vec3 position;
} light;


void main() {
        //f_color = texture(texSampler, frag_tex_coords);
        f_normal = frag_normal;
        f_color = vec4(texture(texSampler, frag_tex_coords).rgb, 1.0);
}
