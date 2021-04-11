#version 450

layout(location = 0) in vec2 f_uv;
layout(location = 1) in vec4 f_color;

layout(location = 0) out vec4 out_color;

layout(set = 0, binding = 0) uniform texture2D t_text;
layout(set = 0, binding = 1) uniform sampler s_text;


void main() {
    out_color = f_color * texture(sampler2D(t_text, s_text), f_uv);
}
