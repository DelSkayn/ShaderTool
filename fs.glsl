#version 330

in vec3 color;
out vec3 frag_color;

uniform vec3 light_pos = vec3(0.5, 10.0,0.5);
in vec3 pos;
in vec3 norm;

void main(){
    vec3 diffuse_color = (color* 0.5 + 0.5);

    vec3 L = normalize(light_pos - pos);
    vec3 N = normalize(norm);
    float lamb = max(dot(N,L), 0.0);
    float spec = 0.0;
    if (lamb > 0.0){
        vec3 R = reflect(-L, N);
        vec3 V = normalize(-pos);
        float spec_angle = max(dot(R,V),0.0);
        spec = pow(spec_angle, 8.0);
    }

    frag_color = diffuse_color + vec3(1.0,1.0,1.0) *  lamb + vec3(1.0,0.0,0.0) * spec;
}
