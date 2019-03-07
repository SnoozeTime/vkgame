#version 450

layout(set=0, binding=0) uniform sampler2D diffuseSampler;

layout(location=0) in vec2 uv;
layout(location=0) out vec4 f_color;

void main() {
        vec4 color = vec4( 0.5 );
        color -= texture( diffuseSampler, uv + vec2( -0.001,  0.0 ) );
        color += texture( diffuseSampler, uv + vec2(  0.001,  0.0 ) );
        color -= texture( diffuseSampler, uv + vec2(  0.0, -0.001 ) );
        color += texture( diffuseSampler, uv + vec2(  0.0,  0.001 ) );


        f_color = abs(0.5 - color);
}
