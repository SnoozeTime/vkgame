#version 450

layout(location=0) in vec2 position;
layout(location=1) in vec2 uv;
layout(location=0) out vec2 outUv;

/*
 *  Draw a quad with texture coordinates to the screen. This is used for post
 *  processing and so.
 * */
void main() {
        outUv = uv;
        gl_Position = vec4(position, 0.0, 1.0);
}
