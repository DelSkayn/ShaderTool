#version 330
#define PI 3.1415926538

uniform float time;

out vec3 frag_color;

float mandelbulb(vec3 p, float power){
    vec3 z = p;
    float dr = 1.0;
    float r = 0.0;

    for (int i = 0;i < 15; ++i){
        r = length(z);
        if (r > 1.5)
            break;

        float theta = acos(z.z / r);
        float phi = atan(z.y, z.x);
        float zr = pow(r,power);
        dr = pow(r,power - 1) * power * dr + 1;

        theta = theta * power;
        phi = phi * power;

        z = zr * vec3(sin(theta) * cos(phi), sin(phi)  * sin(theta), cos(theta));
        z += p;
    }
    return 0.5 * log(r) * r / dr;
}

float distance_sphere(in vec3 p, in vec3 c, float r){
    return length(p - c) - r;
}

float cube(vec3 p, vec3 c)
{
    vec3 q = abs(p) - c;
    return length(max(q,0.0)) + min(max(q.x, max(q.y,q.z)),0.0);
}

float distance_floor(in vec3 p, in float h){


    float height = h + sin(length(p.xz)) * 2.0;

    if (p.y < height){
        return 100000.0;
    }

    return p.y - height;
}

const int STEPS = 100;
const float MIN_HIT = 0.0005;
const float MAX_HIT = 10000.0;

float scene(in vec3 p){
    float dist = MAX_HIT;

    dist = min(dist,distance_sphere(p,vec3(-5.0,0.0,0.0),1.0));
    dist = min(dist,distance_sphere(p,vec3(5.0,0.0,0.0),1.0));
    dist = min(dist,max(cube(p,vec3(2.0)),-distance_sphere(p,vec3(0.0),2.5)));
    dist = min(dist,distance_floor(p,-4.0));
    dist = min(dist,mandelbulb(p,cos(time / 8.0) * 2.0 + 4.0 ));

    return dist;
}

vec3 normal_gradient(in vec3 p){
    const vec2 STEP = vec2(0.001,0.0);

    vec3 gradient = vec3(
            scene(p + STEP.xyy) - scene(p - STEP.xyy),
            scene(p + STEP.yxy) - scene(p - STEP.yxy),
            scene(p + STEP.yyx) - scene(p - STEP.yyx)
    );

    return normalize(gradient);
}


const vec3 GLOW_COLOR = vec3(1.0,0.0,1.0);
const vec3 OTHER_COLOR = vec3(0.0,1.0,1.0);

vec3 ray_march(in vec3 ro, in vec3 rd){
    float dist = 0.0;

    for(int i = 0;i < STEPS;++i){
        vec3 cur_pos = ro + dist * rd;
        float dist_near = scene(cur_pos);
        if (dist_near < MIN_HIT){

            vec3 normal = normal_gradient(cur_pos);
            vec3 light_dir = normalize(vec3(1.1,-1.0,0.0));

            float diffuse = max(0.01,dot(normal,-light_dir));
            vec3 half = normalize(-light_dir - rd);
            float spec_angle = max(dot(half,normal),0.0);
            float specular = pow(spec_angle,64.0);

            float it_factor = smoothstep(0.0,1.0,float(i) / float(STEPS));
            //return vec3(0.0,1.0,1.0) * diffuse + vec3(specular) + 
            return GLOW_COLOR * pow(it_factor,2.0) + OTHER_COLOR * pow(1.0 - it_factor,9.0) * diffuse;// + specular;
        }
        if (dist >= MAX_HIT){
            return GLOW_COLOR * pow(float(i) / float(STEPS),2.0);
        }
        dist += dist_near;
    }
    return GLOW_COLOR;
}

in vec2 text_coord;
in vec4 near;
in vec4 far;

uniform mat4 view;
uniform mat4 projection;

void main(){

    vec3 camera_position = vec3(0.0, 0.0, 0.0);
    vec3 ro = camera_position;
    vec3 rd = vec3(text_coord, 1.0);

    ro = near.xyz/near.w;
    rd = normalize(far.xyz/far.w - ro);

    vec3 shaded_color = ray_march(ro, rd);

    frag_color = shaded_color;
    //frag_color = rd;
    //frag_color = vec3(text_coord,0.0);
}
