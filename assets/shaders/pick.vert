# version 450

layout(location = 0) in vec3 position;
layout(location = 1) in vec2 texcoords;
layout(location = 2) in vec3 normals;
layout(location = 0) out vec4 frag_color;

layout (push_constant) uniform PushConstants {
        vec4 color;
} pushConstants;

layout(binding = 0) uniform Data {
        mat4 model;
        mat4 view;
        mat4 proj;
} uniforms;

void main() {
        frag_color = pushConstants.color;
        gl_Position = uniforms.proj * uniforms.view * uniforms.model * vec4(position, 1.0);
}

