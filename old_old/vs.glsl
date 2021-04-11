#version 450

layout(location = 0) in vec3 position;
layout(location = 1) in vec3 normal;



layout(set = 0, binding = 0) uniform Uniforms{
    mat4 model;
    mat4 view;
    mat4 projection;

};

const vec3 light_pos = vec3(0.5, 10.0,0.5);


void main(){
    color = normal;

    mat4 mv = view * model;
    norm = mat3(mv) * normal;

    vec4 temp_pos = mv * vec4(position, 1.0);
    pos = temp_pos.xyz;
    gl_Position = projection * temp_pos;
}
