#version 300 es
precision highp float;

uniform float u_time;
uniform vec2 u_resolution;
uniform float u_bpm;
uniform float u_beat;
uniform float u_playing;
uniform vec3 u_color1;
uniform vec3 u_color2;
uniform vec3 u_color3;
uniform vec3 u_color4;
out vec4 fragColor;

const float PI  = 3.14159265359;
const float PHI = 1.618033988749895;

// --- Rotation matrices ---
mat3 rotX(float a) { float c=cos(a),s=sin(a); return mat3(1,0,0, 0,c,-s, 0,s,c); }
mat3 rotY(float a) { float c=cos(a),s=sin(a); return mat3(c,0,s, 0,1,0, -s,0,c); }
mat3 rotZ(float a) { float c=cos(a),s=sin(a); return mat3(c,-s,0, s,c,0, 0,0,1); }

// --- Distance from point to line segment ---
float segDist(vec2 p, vec2 a, vec2 b) {
    vec2 pa = p - a, ba = b - a;
    float h = clamp(dot(pa, ba) / dot(ba, ba), 0.0, 1.0);
    return length(pa - ba * h);
}

// --- Symmetric rotation keyframes (Euler angles x,y,z) for each solid ---
// These orientations produce pleasing symmetric projections.
const vec3 SYM[33] = vec3[33](
    // Tetrahedron (7 keyframes, offset 0)
    vec3(0.0,           0.0,            0.0),
    vec3(0.31 * PI,     0.0,            0.75 * PI),
    vec3(0.31 * PI,     0.0,            0.25 * PI),
    vec3(0.25 * PI,     0.0,            0.25 * PI),
    vec3(0.0,          -0.25 * PI,      0.0),
    vec3(0.0,           0.25 * PI,      0.0),
    vec3(0.0,           0.0,            0.25 * PI),
    // Cube (5 keyframes, offset 7)
    vec3(0.0,           0.0,            0.0),
    vec3(0.0,           0.3 * PI,       0.25 * PI),
    vec3(0.0,           0.24 * PI,      0.25 * PI),
    vec3(0.0,           0.125 * PI,     0.25 * PI),
    vec3(0.0,           0.25 * PI,      0.0),
    // Octahedron (9 keyframes, offset 12)
    vec3(0.0,           0.0,            0.0),
    vec3(0.0,           0.5,            0.0),
    vec3(0.12,          PI,             0.0),
    vec3(0.0,           0.25 * PI,      0.25 * PI),
    vec3(0.0,           0.25 * PI,      0.0),
    vec3(0.35 * PI,     0.25 * PI,      0.0),
    vec3(0.25 * PI,     0.25 * PI,      0.0),
    vec3(0.5 * PI,      0.25 * PI,      0.0),
    vec3(0.25 * PI,     0.0,            0.0),
    // Icosahedron (7 keyframes, offset 21)
    vec3(0.0,           0.0,            0.0),
    vec3(0.125 * PI,    0.0,            0.0),
    vec3(0.0,           0.125 * PI,     0.0),
    vec3(0.0,           0.125 * PI,     0.5 * PI),
    vec3(0.0,           0.25 * PI,      0.5 * PI),
    vec3(0.25 * PI,     0.0,            0.0),
    vec3(0.5 * PI,      0.0,            0.0),
    // Dodecahedron (5 keyframes, offset 28)
    vec3(0.0,           0.0,            0.0),
    vec3(0.0,           0.12 * PI,      0.0),
    vec3(0.25 * PI,     0.0,            0.0),
    vec3(0.5 * PI,      PI,             0.24),
    vec3(0.5 * PI,      0.0,            0.0)
);

// --- Cycle through symmetric keyframes with smooth easing & dwell ---
mat3 symRotation(float t, float period, int offset, int count) {
    float phase = t / period;
    int idx = int(mod(floor(phase), float(count)));
    int nxt = int(mod(float(idx + 1), float(count)));
    float f = fract(phase);
    float e = smoothstep(0.0, 1.0, smoothstep(0.12, 0.88, f));
    vec3 a = mix(SYM[offset + idx], SYM[offset + nxt], e);
    return rotX(a.x) * rotY(a.y) * rotZ(a.z);
}

// --- Wireframe glow: core line + inner glow + outer bloom + vertex dots ---
vec3 wireRender(float eDist, float vDist, vec3 color) {
    float line  = smoothstep(0.006, 0.001, eDist);
    float glow  = exp(-eDist * 160.0) * 0.5;
    float bloom = exp(-eDist * 40.0) * 0.12;
    float vert  = smoothstep(0.018, 0.006, vDist) * 1.0;
    float vHalo = exp(-vDist * 80.0) * 0.25;
    return color * (line * 0.7 + glow + bloom) + color * (vert + vHalo);
}

// ============================================================
// TETRAHEDRON  (4 vertices, 6 edges)
// ============================================================
void tetrahedronSolid(vec2 p, mat3 rot, out float eDist, out float vDist) {
    const float S = 0.5773502692; // 1/sqrt(3)
    vec2 v0 = (rot * vec3( S,  S,  S)).xy;
    vec2 v1 = (rot * vec3( S, -S, -S)).xy;
    vec2 v2 = (rot * vec3(-S,  S, -S)).xy;
    vec2 v3 = (rot * vec3(-S, -S,  S)).xy;

    eDist = segDist(p, v0, v1);
    eDist = min(eDist, segDist(p, v0, v2));
    eDist = min(eDist, segDist(p, v0, v3));
    eDist = min(eDist, segDist(p, v1, v2));
    eDist = min(eDist, segDist(p, v1, v3));
    eDist = min(eDist, segDist(p, v2, v3));

    vDist = min(min(length(p-v0), length(p-v1)),
                min(length(p-v2), length(p-v3)));
}

// ============================================================
// CUBE / HEXAHEDRON  (8 vertices, 12 edges)
// ============================================================
void cubeSolid(vec2 p, mat3 rot, out float eDist, out float vDist) {
    const float S = 0.5773502692;
    vec2 v[8];
    v[0] = (rot * vec3( S,  S,  S)).xy;
    v[1] = (rot * vec3( S,  S, -S)).xy;
    v[2] = (rot * vec3( S, -S,  S)).xy;
    v[3] = (rot * vec3( S, -S, -S)).xy;
    v[4] = (rot * vec3(-S,  S,  S)).xy;
    v[5] = (rot * vec3(-S,  S, -S)).xy;
    v[6] = (rot * vec3(-S, -S,  S)).xy;
    v[7] = (rot * vec3(-S, -S, -S)).xy;

    eDist = segDist(p, v[0], v[1]);
    eDist = min(eDist, segDist(p, v[0], v[2]));
    eDist = min(eDist, segDist(p, v[0], v[4]));
    eDist = min(eDist, segDist(p, v[1], v[3]));
    eDist = min(eDist, segDist(p, v[1], v[5]));
    eDist = min(eDist, segDist(p, v[2], v[3]));
    eDist = min(eDist, segDist(p, v[2], v[6]));
    eDist = min(eDist, segDist(p, v[3], v[7]));
    eDist = min(eDist, segDist(p, v[4], v[5]));
    eDist = min(eDist, segDist(p, v[4], v[6]));
    eDist = min(eDist, segDist(p, v[5], v[7]));
    eDist = min(eDist, segDist(p, v[6], v[7]));

    vDist = length(p - v[0]);
    for (int i = 1; i < 8; i++) vDist = min(vDist, length(p - v[i]));
}

// ============================================================
// OCTAHEDRON  (6 vertices, 12 edges)
// ============================================================
void octahedronSolid(vec2 p, mat3 rot, out float eDist, out float vDist) {
    vec2 v[6];
    v[0] = (rot * vec3( 1, 0, 0)).xy;
    v[1] = (rot * vec3(-1, 0, 0)).xy;
    v[2] = (rot * vec3( 0, 1, 0)).xy;
    v[3] = (rot * vec3( 0,-1, 0)).xy;
    v[4] = (rot * vec3( 0, 0, 1)).xy;
    v[5] = (rot * vec3( 0, 0,-1)).xy;

    // 12 edges: every pair except antipodal (0-1, 2-3, 4-5)
    eDist = segDist(p, v[0], v[2]);
    eDist = min(eDist, segDist(p, v[0], v[3]));
    eDist = min(eDist, segDist(p, v[0], v[4]));
    eDist = min(eDist, segDist(p, v[0], v[5]));
    eDist = min(eDist, segDist(p, v[1], v[2]));
    eDist = min(eDist, segDist(p, v[1], v[3]));
    eDist = min(eDist, segDist(p, v[1], v[4]));
    eDist = min(eDist, segDist(p, v[1], v[5]));
    eDist = min(eDist, segDist(p, v[2], v[4]));
    eDist = min(eDist, segDist(p, v[2], v[5]));
    eDist = min(eDist, segDist(p, v[3], v[4]));
    eDist = min(eDist, segDist(p, v[3], v[5]));

    vDist = length(p - v[0]);
    for (int i = 1; i < 6; i++) vDist = min(vDist, length(p - v[i]));
}

// ============================================================
// ICOSAHEDRON  (12 vertices, 30 edges)
// ============================================================
void icosahedronSolid(vec2 p, mat3 rot, out float eDist, out float vDist) {
    // Normalized to unit sphere: N = 1/sqrt(1+PHI^2), M = PHI*N
    const float N = 0.5257311121;
    const float M = 0.8506508084;

    vec2 v[12];
    v[0]  = (rot * vec3( 0,  N,  M)).xy;
    v[1]  = (rot * vec3( 0,  N, -M)).xy;
    v[2]  = (rot * vec3( 0, -N,  M)).xy;
    v[3]  = (rot * vec3( 0, -N, -M)).xy;
    v[4]  = (rot * vec3( N,  M,  0)).xy;
    v[5]  = (rot * vec3( N, -M,  0)).xy;
    v[6]  = (rot * vec3(-N,  M,  0)).xy;
    v[7]  = (rot * vec3(-N, -M,  0)).xy;
    v[8]  = (rot * vec3( M,  0,  N)).xy;
    v[9]  = (rot * vec3( M,  0, -N)).xy;
    v[10] = (rot * vec3(-M,  0,  N)).xy;
    v[11] = (rot * vec3(-M,  0, -N)).xy;

    // 30 edges
    // v0 neighbors: 2,4,6,8,10
    eDist = segDist(p, v[0], v[2]);
    eDist = min(eDist, segDist(p, v[0], v[4]));
    eDist = min(eDist, segDist(p, v[0], v[6]));
    eDist = min(eDist, segDist(p, v[0], v[8]));
    eDist = min(eDist, segDist(p, v[0], v[10]));
    // v1 neighbors: 3,4,6,9,11
    eDist = min(eDist, segDist(p, v[1], v[3]));
    eDist = min(eDist, segDist(p, v[1], v[4]));
    eDist = min(eDist, segDist(p, v[1], v[6]));
    eDist = min(eDist, segDist(p, v[1], v[9]));
    eDist = min(eDist, segDist(p, v[1], v[11]));
    // v2 neighbors (new): 5,7,8,10
    eDist = min(eDist, segDist(p, v[2], v[5]));
    eDist = min(eDist, segDist(p, v[2], v[7]));
    eDist = min(eDist, segDist(p, v[2], v[8]));
    eDist = min(eDist, segDist(p, v[2], v[10]));
    // v3 neighbors (new): 5,7,9,11
    eDist = min(eDist, segDist(p, v[3], v[5]));
    eDist = min(eDist, segDist(p, v[3], v[7]));
    eDist = min(eDist, segDist(p, v[3], v[9]));
    eDist = min(eDist, segDist(p, v[3], v[11]));
    // v4 neighbors (new): 6,8,9
    eDist = min(eDist, segDist(p, v[4], v[6]));
    eDist = min(eDist, segDist(p, v[4], v[8]));
    eDist = min(eDist, segDist(p, v[4], v[9]));
    // v5 neighbors (new): 7,8,9
    eDist = min(eDist, segDist(p, v[5], v[7]));
    eDist = min(eDist, segDist(p, v[5], v[8]));
    eDist = min(eDist, segDist(p, v[5], v[9]));
    // v6 neighbors (new): 10,11
    eDist = min(eDist, segDist(p, v[6], v[10]));
    eDist = min(eDist, segDist(p, v[6], v[11]));
    // v7 neighbors (new): 10,11
    eDist = min(eDist, segDist(p, v[7], v[10]));
    eDist = min(eDist, segDist(p, v[7], v[11]));
    // remaining: 8-9, 10-11
    eDist = min(eDist, segDist(p, v[8], v[9]));
    eDist = min(eDist, segDist(p, v[10], v[11]));

    vDist = length(p - v[0]);
    for (int i = 1; i < 12; i++) vDist = min(vDist, length(p - v[i]));
}

// ============================================================
// DODECAHEDRON  (20 vertices, 30 edges)
// ============================================================
void dodecahedronSolid(vec2 p, mat3 rot, out float eDist, out float vDist) {
    // All vertices at distance sqrt(3) from origin; normalize by 1/sqrt(3)
    const float S = 0.5773502692;            // 1/sqrt(3)
    const float A = 0.3568220898;            // (1/PHI)/sqrt(3)
    const float B = 0.9341723590;            // PHI/sqrt(3)

    vec2 v[20];
    // Cube-type (±1,±1,±1)/sqrt(3)
    v[0]  = (rot * vec3( S,  S,  S)).xy;
    v[1]  = (rot * vec3( S,  S, -S)).xy;
    v[2]  = (rot * vec3( S, -S,  S)).xy;
    v[3]  = (rot * vec3( S, -S, -S)).xy;
    v[4]  = (rot * vec3(-S,  S,  S)).xy;
    v[5]  = (rot * vec3(-S,  S, -S)).xy;
    v[6]  = (rot * vec3(-S, -S,  S)).xy;
    v[7]  = (rot * vec3(-S, -S, -S)).xy;
    // (0, ±PHI, ±1/PHI)/sqrt(3)
    v[8]  = (rot * vec3( 0,  B,  A)).xy;
    v[9]  = (rot * vec3( 0,  B, -A)).xy;
    v[10] = (rot * vec3( 0, -B,  A)).xy;
    v[11] = (rot * vec3( 0, -B, -A)).xy;
    // (±1/PHI, 0, ±PHI)/sqrt(3)
    v[12] = (rot * vec3( A,  0,  B)).xy;
    v[13] = (rot * vec3( A,  0, -B)).xy;
    v[14] = (rot * vec3(-A,  0,  B)).xy;
    v[15] = (rot * vec3(-A,  0, -B)).xy;
    // (±PHI, ±1/PHI, 0)/sqrt(3)
    v[16] = (rot * vec3( B,  A,  0)).xy;
    v[17] = (rot * vec3( B, -A,  0)).xy;
    v[18] = (rot * vec3(-B,  A,  0)).xy;
    v[19] = (rot * vec3(-B, -A,  0)).xy;

    // 24 cube-to-rectangle edges
    eDist = segDist(p, v[0], v[8]);
    eDist = min(eDist, segDist(p, v[0], v[12]));
    eDist = min(eDist, segDist(p, v[0], v[16]));
    eDist = min(eDist, segDist(p, v[1], v[9]));
    eDist = min(eDist, segDist(p, v[1], v[13]));
    eDist = min(eDist, segDist(p, v[1], v[16]));
    eDist = min(eDist, segDist(p, v[2], v[10]));
    eDist = min(eDist, segDist(p, v[2], v[12]));
    eDist = min(eDist, segDist(p, v[2], v[17]));
    eDist = min(eDist, segDist(p, v[3], v[11]));
    eDist = min(eDist, segDist(p, v[3], v[13]));
    eDist = min(eDist, segDist(p, v[3], v[17]));
    eDist = min(eDist, segDist(p, v[4], v[8]));
    eDist = min(eDist, segDist(p, v[4], v[14]));
    eDist = min(eDist, segDist(p, v[4], v[18]));
    eDist = min(eDist, segDist(p, v[5], v[9]));
    eDist = min(eDist, segDist(p, v[5], v[15]));
    eDist = min(eDist, segDist(p, v[5], v[18]));
    eDist = min(eDist, segDist(p, v[6], v[10]));
    eDist = min(eDist, segDist(p, v[6], v[14]));
    eDist = min(eDist, segDist(p, v[6], v[19]));
    eDist = min(eDist, segDist(p, v[7], v[11]));
    eDist = min(eDist, segDist(p, v[7], v[15]));
    eDist = min(eDist, segDist(p, v[7], v[19]));
    // 6 rectangle-to-rectangle edges
    eDist = min(eDist, segDist(p, v[8],  v[9]));
    eDist = min(eDist, segDist(p, v[10], v[11]));
    eDist = min(eDist, segDist(p, v[12], v[14]));
    eDist = min(eDist, segDist(p, v[13], v[15]));
    eDist = min(eDist, segDist(p, v[16], v[17]));
    eDist = min(eDist, segDist(p, v[18], v[19]));

    vDist = length(p - v[0]);
    for (int i = 1; i < 20; i++) vDist = min(vDist, length(p - v[i]));
}

// ============================================================
void main() {
    vec2 uv = (gl_FragCoord.xy - 0.5 * u_resolution) / min(u_resolution.x, u_resolution.y);

    float tempo = u_bpm / 120.0;
    float t = mix(u_time * 0.5 * tempo, u_beat * 0.5, u_playing);

    // Scale: unit-sphere solids at ~40% of screen height
    vec2 p = uv / 0.38;

    // --- Build rotation matrices from symmetric keyframes ---
    // Each solid cycles through its own set of pleasing symmetric
    // orientations at different periods, so the overlapping contours
    // keep shifting.
    mat3 rTetra = symRotation(t, 3.5,  0, 7);   // tetrahedron: 7 keyframes
    mat3 rCube  = symRotation(t, 4.0,  7, 5);   // cube:        5 keyframes
    mat3 rOcta  = symRotation(t, 2.8, 12, 9);   // octahedron:  9 keyframes
    mat3 rIcosa = symRotation(t, 3.2, 21, 7);   // icosahedron: 7 keyframes
    mat3 rDodec = symRotation(t, 5.0, 28, 5);   // dodecahedron:5 keyframes

    // --- Compute distances ---
    float eT, vT, eC, vC, eO, vO, eI, vI, eD, vD;
    tetrahedronSolid (p, rTetra, eT, vT);
    cubeSolid        (p, rCube,  eC, vC);
    octahedronSolid  (p, rOcta,  eO, vO);
    icosahedronSolid (p, rIcosa, eI, vI);
    dodecahedronSolid(p, rDodec, eD, vD);

    // --- Color assignment: 4 theme colors across 5 solids ---
    vec3 cTetra = u_color1;
    vec3 cCube  = u_color2;
    vec3 cOcta  = u_color3;
    vec3 cIcosa = mix(u_color1, u_color3, 0.5);
    vec3 cDodec = mix(u_color2, u_color3, 0.5);

    // --- Composite wireframes (additive) ---
    vec3 col = vec3(0.0);
    col += wireRender(eT, vT, cTetra);
    col += wireRender(eC, vC, cCube);
    col += wireRender(eO, vO, cOcta);
    col += wireRender(eI, vI, cIcosa) * 0.7;  // complex solids slightly dimmer
    col += wireRender(eD, vD, cDodec) * 0.7;

    // --- Beat pulse: brief brightness surge on each beat ---
    float beatPhase = fract(mix(t * tempo, u_beat, u_playing));
    float pulse = exp(-beatPhase * 6.0) * 0.2;
    col *= 1.0 + pulse;

    // --- Subtle background tint from color4 ---
    col += u_color4 * 0.03;

    // --- Vignette ---
    float vig = 1.0 - dot(uv, uv) * 0.5;
    col *= max(vig, 0.0);

    // --- Scanlines ---
    float scan = 0.92 + 0.08 * sin(gl_FragCoord.y * 2.5);
    col *= scan;

    fragColor = vec4(col, 1.0);
}
