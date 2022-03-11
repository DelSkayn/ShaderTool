#version 330

in vec3 position;

out vec4 far;
out vec4 near;

uniform mat4 model;
uniform mat4 view;
uniform mat4 projection;

void main(){
    mat4 mvp = projection * view;
    mat4 mvp_inv = inverse(mvp);

    gl_Position = vec4(position, 1.0);

    near = mvp_inv * vec4(position.xy,-1.0,1.0);
    far =  mvp_inv * vec4(position.xy,1.0,1.0);
}
