#version 450

layout(set=0, binding=0) uniform sampler2D diffuseSampler;

layout(location=0) in vec2 uv;
layout(location=0) out vec4 f_color;

float greyscale(vec3 color) {
        return  0.299 * color.x + 0.587 * color.y + 0.114 * color.z;
}

void main() {

        mat3 kernel_vert;
        kernel_vert[0] = vec3(-1.0, 0.0, 1.0);
        kernel_vert[1] = vec3(-2.0, 0.0, 2.0);
        kernel_vert[2] = vec3(-1.0, 0.0, 1.0);
        vec2 image_size = textureSize(diffuseSampler, 0);
        float edge = 1/ image_size.x;
        float edge_x = 1.0/ image_size.x;
        float edge_y = 1.0/ image_size.x;
        float edge_vert =0.0;
        for (int i = 0; i < 3; i++) {

                for (int j = 0; j < 3; j++) {
                        edge_vert += kernel_vert[i][j] * greyscale(texture(diffuseSampler, uv + vec2((i-1)*edge_x, (j-1)*edge_y)).rgb);
                }
        }

        mat3 kernel_horiz;
        kernel_horiz[0] = vec3(-1.0, -2.0, -1.0);
        kernel_horiz[1] = vec3(0.0, 0.0, 0.0);
        kernel_horiz[2] = vec3(1.0, 2.0, 1.0);
        float edge_horiz = 0.0;
        for (int i = 0; i < 3; i++) {

                for (int j = 0; j < 3; j++) {
                        edge_horiz += kernel_horiz[i][j] * greyscale(texture(diffuseSampler, uv + vec2((i-1)*edge_x, (j-1)*edge_y)).rgb);
                }
        }
        float filter_value = sqrt(edge_vert*edge_vert + edge_horiz*edge_horiz);

        if (filter_value > 0.01) {
                f_color = vec4(0.0, 0.0, 0.0, 1.0);
        } else {
                discard;
        }
}
