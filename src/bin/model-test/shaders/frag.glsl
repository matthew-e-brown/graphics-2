#version 460 core

#define MAX_LIGHTS 8

struct Material {
    vec3 diffuse;
    vec3 ambient;
    vec3 specular;
    float specPow;
    float alpha;
};

struct Light {
    vec3 diffuse;
    vec3 ambient;
    vec3 specular;
    vec3 position;
};

uniform Material uMaterial;
uniform Light uLights[MAX_LIGHTS];
uniform int uNumLights = 0;

in vec3 vPosition;
in vec3 vNormal;
in vec2 vTexCoord;

out vec4 fColor;


vec3 blinnPhong(Material material, Light light) {
    vec3 L = normalize(light.position - vPosition);
    vec3 E = normalize(-vPosition); // in camera space, eye is at the origin
    vec3 H = normalize(L + E);
    vec3 N = normalize(vNormal);

    vec3 dProd = light.diffuse * material.diffuse;
    vec3 aProd = light.ambient * material.ambient;
    vec3 sProd = light.specular * material.specular;

    float Kd = max( dot(L, N), 0.0 );
    float Ks = max( dot(H, N), 0.0 );
    Ks = pow(Ks, material.specPow);

    vec3 diff = dProd * Kd;
    vec3 spec = sProd * Ks;

    if (dot(L, N) < 0.0) {
        spec = vec3(0.0);
    }

    return aProd + diff + spec;
}


void main() {
    fColor.xyz = vec3(0.0);

    for (int i = 0; i < uNumLights && i < MAX_LIGHTS; i++) {
        fColor.xyz += blinnPhong(uMaterial, uLights[i]);
    }

    fColor.a = 1.0;
}
