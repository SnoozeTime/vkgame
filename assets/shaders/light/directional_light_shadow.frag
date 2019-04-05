#version 450

// These are the diffuse, normals and depth that we have renderer to some buffers the
// previous render subpass.
layout(input_attachment_index = 0, set = 0, binding = 0) uniform subpassInput u_diffuse;
layout(input_attachment_index = 1, set = 0, binding = 1) uniform subpassInput u_normals;
layout(input_attachment_index = 2, set = 0, binding = 2) uniform subpassInput u_depth;
layout(input_attachment_index = 3, set = 0, binding = 3) uniform subpassInput u_position;
layout(input_attachment_index = 4, set = 0, binding = 4) uniform subpassInput u_shadow;
layout(set = 0, binding = 5) uniform Data {
        mat4 view;
        mat4 proj;
} light_vp;

// For the point light
layout(push_constant) uniform PushConstants {
        vec4 color;
        vec4 position;
} push_constants;

layout(location = 0) in vec2 v_screen_coords;
layout(location = 0) out vec4 f_color;

float shadow_factor() {
        
        vec4 world_pos = subpassLoad(u_position);
        vec4 frag_pos_from_lightpov = light_vp.proj * light_vp.view * world_pos;

        vec3 frag_pos_clip = vec3(frag_pos_from_lightpov.x,
                                  frag_pos_from_lightpov.y,
                                  frag_pos_from_lightpov.z/frag_pos_from_lightpov.w);
        vec3 first_obj_pos = subpassLoad(u_shadow).rgb;

        if (first_obj_pos.r < frag_pos_clip.z) {
                return 0.0;
        }
        return 0.0;
}

void main() {
        float in_depth = subpassLoad(u_depth).x;
        // Any depth superior or equal to 1.0 means that the pixel has been untouched by the deferred
        // pass. We don't want to deal with them.
        if (in_depth >= 1.0) {
//                f_color = vec4(1.0, 1.0, 1.0, 1.0);
                return;
        }

        
        vec3 norm = normalize(subpassLoad(u_normals).rgb);
        vec3 lightDir = normalize(push_constants.position.xyz);
        float diff = max(dot(norm, lightDir), 0.0);

        vec3 diffuse = (1.0 - shadow_factor()) * diff * push_constants.color.rgb * subpassLoad(u_diffuse).rgb;
        f_color = vec4(diffuse, 1.0);
}
