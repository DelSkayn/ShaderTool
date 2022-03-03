#version 330

out vec3 frag_color;



in vec2 text_coord;

void main(){
    frag_color = vec3(text_coord,0.0);
}
