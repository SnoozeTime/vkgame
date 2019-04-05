#version 450
layout(location=0) in vec2 inUv;
layout(set=1, binding=0) uniform sampler2D texSampler;

layout(location = 0) out vec4 f_color;
// nothing to see here
void main() {
        f_color = vec4(texture(texSampler, inUv).rgb, 1.0);
}
