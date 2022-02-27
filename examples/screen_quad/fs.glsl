#version 330

uniform float multiplier = 40.0;
uniform float time;
uniform vec2 mouse_pos;
uniform vec2 window_size;

in vec2 uv;
out vec4 color;

void main(){
    vec2 middle = uv * 2.0 - 1.0;

    vec2 mouse = mouse_pos / window_size;

    float x = (sin(abs(middle.x) * multiplier + time * (1.0 + mouse.x)) + 1.0) / 2.0;
    float y = (cos(time * (1.0 +  mouse.y) + length(middle) * multiplier) + 1.0) / 2.0;

    color = vec4(x,y,0.0,1.0);
}
