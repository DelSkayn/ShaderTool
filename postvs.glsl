#version 330

in vec3 position;
in vec3 normal;

uniform mat4 model;
uniform mat4 view;
uniform mat4 projection;

out vec2 pos;

void main(){
    pos = position.xy;
    gl_Position = vec4(position.xy,0.99999,1.0);
}
