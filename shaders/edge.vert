#version 450

layout(location=0) in vec2 position;
layout(location=0) out vec2 uv;

out gl_PerVertex {
        vec4 gl_Position;
};

void main() {
        uv = position;
        gl_Position = vec4(position, 0.0, 1.0);
}
