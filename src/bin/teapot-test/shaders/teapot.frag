#version 460 core

#define MAX_LIGHTS 16

in vec3 vPosition;
in vec3 vNormal;

struct Material {
    vec4 diffuse;
    vec4 ambient;
    vec4 specular;
    float shininess;
};

struct Light {
    vec4 diffuse;
    vec4 ambient;
    vec4 specular;
    vec3 position;
};

uniform Material material;
uniform Light lights[MAX_LIGHTS];
uniform int numLights = 0;

out vec4 fColor;


vec4 bpShading(Material material, Light light) {
    vec3 L = normalize(light.position - vPosition);
    vec3 E = normalize(-vPosition); // in camera space, eye is at the origin
    vec3 H = normalize(L + E);
    vec3 N = normalize(vNormal);

    vec4 dProd = light.diffuse * material.diffuse;
    vec4 aProd = light.ambient * material.ambient;
    vec4 sProd = light.specular * material.specular;

    float Kd = max( dot(L, N), 0.0 );
    float Ks = max( dot(H, N), 0.0 );
    Ks = pow(Ks, material.shininess);

    vec4 diff = dProd * Kd;
    vec4 spec = sProd * Ks;

    if (dot(L, N) < 0.0) {
        spec = vec4(0.0, 0.0, 0.0, 1.0);
    }

    return aProd + diff + spec;
}


void main() {
    fColor = vec4(0.0);

    for (int i = 0; i < numLights && i < MAX_LIGHTS; i++) {
        fColor += bpShading(material, lights[i]);
    }
    fColor.a = 1.0;
}
