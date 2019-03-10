#version 450


layout(location = 0) in vec3 frag_tex_coords;

layout(location=0) out vec4 f_color;

layout(set = 1, binding = 0) uniform samplerCube cubetex;

void main() {
        f_color = texture(cubetex, frag_tex_coords);
}
