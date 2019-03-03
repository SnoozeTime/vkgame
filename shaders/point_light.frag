#version 450

// These are the diffuse, normals and depth that we have renderer to some buffers the
// previous render subpass.
layout(set = 0, binding = 0) uniform sampler2D u_diffuse;
//layout(set = 0, binding = 1) uniform sampler2D u_normals;
//layout(set = 0, binding = 2) uniform sampler2D u_frag_pos;
//layout(set = 0, binding = 3) uniform sampler2D u_depth;

// For the point light
layout(push_constant) uniform PushConstants {
        vec4 color;
        vec4 position;
} push_constants;

layout(location = 0) in vec2 uv;
layout(location = 0) out vec4 f_color;

void main() {
        f_color = texture(u_diffuse, uv);
//        float in_depth = subpassLoad(u_depth).x;
//        // Any depth superior or equal to 1.0 means that the pixel has been untouched by the deferred
//        // pass. We don't want to deal with them.
//        if (in_depth >= 1.0) {
//                f_color = vec4(subpassLoad(u_diffuse).rgb, 1.0);
//                return;
//        }
//
//        vec3 norm = normalize(subpassLoad(u_normals).rgb);
//        vec3 lightDir = normalize(push_constants.position.xyz - subpassLoad(u_frag_pos).rgb);
//        float diff = max(dot(norm, lightDir), 0.0);
//
//        float light_distance = length(push_constants.position.xyz - subpassLoad(u_frag_pos).xyz);
//        // Further decrease light_percent based on the distance with the light position.
////        diff *= 1.0 / exp(light_distance);
//
//        vec3 diffuse = diff * push_constants.color.rgb * subpassLoad(u_diffuse).rgb;
//        f_color = vec4(diffuse, 1.0);
}
