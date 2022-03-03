#version 330

in vec3 position;
in vec2 tex_coord;

out vec2 text_coord;

void main(){
    text_coord = tex_coord;
    gl_Position = vec4(position, 1.0);
}
