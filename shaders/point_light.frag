#version 450

// These are the diffuse, normals and depth that we have renderer to some buffers the
// previous render subpass.
layout(input_attachment_index = 0, set = 0, binding = 0) uniform subpassInput u_diffuse;
layout(input_attachment_index = 1, set = 0, binding = 1) uniform subpassInput u_normals;
layout(input_attachment_index = 2, set = 0, binding = 2) uniform subpassInput u_frag_pos;
layout(input_attachment_index = 3, set = 0, binding = 3) uniform subpassInput u_depth;

// For the point light
layout(push_constant) uniform PushConstants {
        vec4 color;
        vec4 position;
} push_constants;

layout(location = 0) in vec2 v_screen_coords;
layout(location = 0) out vec4 f_color;

void main() {
        float in_depth = subpassLoad(u_depth).x;
        // Any depth superior or equal to 1.0 means that the pixel has been untouched by the deferred
        // pass. We don't want to deal with them.
        if (in_depth >= 1.0) {
                discard;
        }

        vec3 norm = normalize(subpassLoad(u_normals).rgb);
        vec3 lightDir = normalize(push_constants.position.xyz - subpassLoad(u_frag_pos).rgb);
        float diff = max(dot(norm, lightDir), 0.0);
        vec3 diffuse = diff * push_constants.color.rgb * subpassLoad(u_diffuse).rgb;
        // // Find the world coordinates of the current pixel.
        // vec4 world = push_constants.screen_to_world * vec4(v_screen_coords, in_depth, 1.0);
        // world /= world.w;

        // vec3 in_normal = normalize(subpassLoad(u_normals).rgb);
        // vec3 light_direction = normalize(push_constants.position.xyz - world.xyz);
        // // Calculate the percent of lighting that is received based on the orientation of the normal
        // // and the direction of the light.
        // float light_percent = max(-dot(light_direction, in_normal), 0.0);

        // float light_distance = length(push_constants.position.xyz - world.xyz);
        // // Further decrease light_percent based on the distance with the light position.
        // //light_percent *= 1.0 / exp(light_distance);

        // vec3 in_diffuse = subpassLoad(u_diffuse).rgb;
        // f_color.rgb = push_constants.color.rgb * light_percent * in_diffuse;
        // f_color.a = 1.0;
        f_color = vec4(diffuse, 1.0);
}
