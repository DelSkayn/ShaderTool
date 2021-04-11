#version 330

in vec3 position;
in vec3 normal;

uniform mat4 model;
uniform mat4 view;
uniform mat4 projection;

out vec3 color;
out vec3 pos;
out vec3 norm;

void main(){
    color = normal;

    mat4 mv = view * model;
    norm = mat3(mv) * normal;

    vec4 temp_pos = mv * vec4(position, 1.0);
    pos = temp_pos.xyz;
    gl_Position = projection * temp_pos;
}
