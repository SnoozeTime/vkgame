#version 450


layout (location = 0) in vec3 position;

layout (binding = 0) uniform UBO 
{
        mat4 projection;
        mat4 view;
        mat4 model;
} ubo;

layout (location = 0) out vec3 outUVW;


void main() {
        outUVW = position;
        gl_Position = ubo.projection * ubo.view * ubo.model * vec4(position, 1.0);
}

