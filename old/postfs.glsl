#version 330

in vec2 pos;
out vec3 frag_color;

uniform mat4 view;


void main(){
    frag_color = vec3(pos,0.1);
}
