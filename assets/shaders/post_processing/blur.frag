#version 450

layout(set=0, binding=0) uniform sampler2D diffuseSampler;

layout(location=0) in vec2 uv;
layout(location=0) out vec4 f_color;

void main() {

        mat3 kernel;
        kernel[0] = vec3(1.0, 1.0, 1.0);
        kernel[1] = vec3(1.0, 1.0, 1.0);
        kernel[2] = vec3(1.0, 1.0, 1.0);
        vec4 color = vec4( 0.5 );
        vec2 image_size = textureSize(diffuseSampler, 0);
        float edge = 1/ image_size.x;
        float edge_x = 1.0/ image_size.x;
        float edge_y = 1.0/ image_size.x;
        vec4 blur = vec4(0.0);
        for (int i = 0; i < 3; i++) {

                for (int j = 0; j < 3; j++) {
                        blur += kernel[i][j] * texture(diffuseSampler, uv + vec2((i-1)*edge_x, (j-1)*edge_y));
                }
        }
        f_color = blur / 9.0; //abs(0.5 - color);
}
