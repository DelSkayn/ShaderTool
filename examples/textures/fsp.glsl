#version 330

in vec3 color;
out vec3 col_target;

uniform mat4 view;

uniform vec3 light_pos = vec3(3.5, 20.0,0.5);

uniform sampler2D texture_cat;
in vec3 pos;
in vec3 norm;
in vec2 text_coord;

void main(){
    vec3 c = texture(texture_cat,text_coord).rgb;
    vec3 diffuse_color = c * (norm + 1.0) / 2.0;
    vec3 light_pos = mat3(view) * light_pos;

    c = normalize(c);

    vec3 L = normalize(light_pos - pos);
    vec3 N = normalize((norm + c) / 2);
    float lamb = max(dot(N,L), 0.0);
    float spec = 0.0;
    if (lamb > 0.0){
        vec3 R = reflect(-L, N);
        float spec_angle = max(-dot(R,normalize(pos)),0.0);
        spec = pow(spec_angle, 32.0);
    }

    col_target = diffuse_color * 0.1 + diffuse_color *  lamb + diffuse_color * spec;
}
