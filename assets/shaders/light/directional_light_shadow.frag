#version 450

// These are the diffuse, normals and depth that we have renderer to some buffers the
// previous render subpass.
layout(input_attachment_index = 0, set = 0, binding = 0) uniform subpassInput u_diffuse;
layout(input_attachment_index = 1, set = 0, binding = 1) uniform subpassInput u_normals;
layout(input_attachment_index = 2, set = 0, binding = 2) uniform subpassInput u_depth;
layout(input_attachment_index = 3, set = 0, binding = 3) uniform subpassInput u_position;
layout(set = 0, binding = 4) uniform sampler2D u_shadow;
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
        
        vec4 world_pos = vec4(subpassLoad(u_position).rgb, 1.0);
        vec4 shadow_clip = light_vp.view * world_pos;
        float shadow = 1.0;

        vec4 shadowCoord = shadow_clip / shadow_clip.w;
        //shadowCoord.st = shadowCoord.st * 0.5 + 0.5;


        if (shadowCoord.z > -1.0 && shadowCoord.z < 1.0) {
                float dist = texture(u_shadow, shadowCoord.st).r;
                if (shadowCoord.w > 0.0 && dist < shadowCoord.z) {
                        shadow = 0.25;
                }
        }

        return shadow;
}

void main() {
        float in_depth = subpassLoad(u_depth).x;
        // Any depth superior or equal to 1.0 means that the pixel has been untouched by the deferred
        // pass. We don't want to deal with them.
        if (in_depth >= 1.0) {
                return;
        }
        
        vec3 norm = normalize(subpassLoad(u_normals).rgb);
        vec3 lightDir = normalize(push_constants.position.xyz);
        float diff = max(dot(norm, lightDir), 0.0);

        vec3 diffuse = shadow_factor() * diff * push_constants.color.rgb * subpassLoad(u_diffuse).rgb;
        f_color = vec4(diffuse, 1.0);
}
