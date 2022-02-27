#version 330
in vec3 position;
in vec2 tex_coord;

out vec2 uv;

void main(){
    gl_Position = vec4(position,1.0);
    uv = tex_coord;
}
