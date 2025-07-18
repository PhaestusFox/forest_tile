struct Vertex {
    @builtin(vertex_index) vertex_index: u32,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) offset: vec2<f32>,
};

struct View {
    clip_from_world: mat4x4<f32>,
    unjittered_clip_from_world: mat4x4<f32>,
    world_from_clip: mat4x4<f32>,
    world_from_view: mat4x4<f32>,
    view_from_world: mat4x4<f32>,
    // Typically a column-major right-handed projection matrix, one of either:
    //
    // Perspective (infinite reverse z)
    // ```
    // f = 1 / tan(fov_y_radians / 2)
    //
    // ⎡ f / aspect  0   0     0 ⎤
    // ⎢          0  f   0     0 ⎥
    // ⎢          0  0   0  near ⎥
    // ⎣          0  0  -1     0 ⎦
    // ```
    //
    // Orthographic
    // ```
    // w = right - left
    // h = top - bottom
    // d = far - near
    // cw = -right - left
    // ch = -top - bottom
    //
    // ⎡ 2 / w      0      0   cw / w ⎤
    // ⎢     0  2 / h      0   ch / h ⎥
    // ⎢     0      0  1 / d  far / d ⎥
    // ⎣     0      0      0        1 ⎦
    // ```
    //
    // `clip_from_view[3][3] == 1.0` is the standard way to check if a projection is orthographic
    //
    // Wgsl matrices are column major, so for example getting the near plane of a perspective projection is `clip_from_view[3][2]`
    // 
    // Custom projections are also possible however.
    clip_from_view: mat4x4<f32>,
    view_from_clip: mat4x4<f32>,
    world_position: vec3<f32>,
    exposure: f32,
    // viewport(x_origin, y_origin, width, height)
    viewport: vec4<f32>,
    // 6 world-space half spaces (normal: vec3, distance: f32) ordered left, right, top, bottom, near, far.
    // The normal vectors point towards the interior of the frustum.
    // A half space contains `p` if `normal.dot(p) + distance > 0.`
    frustum: array<vec4<f32>, 6>,
    color_grading: ColorGrading,
    mip_bias: f32,
    frame_count: u32,
};

struct ColorGrading {
    balance: mat3x3<f32>,
    saturation: vec3<f32>,
    contrast: vec3<f32>,
    gamma: vec3<f32>,
    gain: vec3<f32>,
    lift: vec3<f32>,
    midtone_range: vec2<f32>,
    exposure: f32,
    hue: f32,
    post_saturation: f32,
}

@group(0) @binding(0) var<uniform> view: View;

@group(2) @binding(0) var base_color_texture: texture_2d<f32>;
@group(2) @binding(1) var base_color_sampler: sampler;
@group(2) @binding(2) var noise_texture: texture_2d_array<f32>;
@group(2) @binding(3) var decor_array: texture_2d_array<f32>;
@group(2) @binding(4) var decor_array_sampler: sampler;


@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    let cell_size = textureDimensions(base_color_texture);
    let cell_w = ceil(view.viewport.z / f32(cell_size.x));
    let cell_h = ceil(view.viewport.w / f32(cell_size.y));
    var x = (i32(mesh.uv.x) % i32(cell_size.x));
    var y = (i32(mesh.uv.y) % i32(cell_size.y));


    var c_x = view.world_position.x / f32(cell_size.x);
    var c_y = view.world_position.y / f32(cell_size.y);
    var cell_x = mesh.uv.x / f32(cell_size.x);
    var cell_y = mesh.uv.y / f32(cell_size.y);

    var l_f = fract(cell_x) < 0.5;
    var b_f = fract(cell_y) < 0.5;

    var l_c = fract(c_x) < 0.5;
    var b_c = fract(c_y) < 0.5;

    cell_x = floor(cell_x);
    cell_y = floor(cell_y);
    c_x = floor(c_x);
    c_y = floor(c_y);

    let x_d = (cell_x- c_x);
    let y_d = (cell_y - c_y);

    let local_x = floor((cell_x - c_x) + (cell_w / 2.)) + 0.5;
    let local_y = floor((cell_y - c_y) + (cell_h / 2.)) + 0.5;
    
    // let n = find_edge(i32(mesh.uv.x), i32(mesh.uv.y));
    // if true {
    //     var out = vec4<f32>(n, n, n, 1.0);
    //     if n < 0. {
    //      out.r = 1.;
    //     }
    //     if n > 1. {
    //         out.b = 0.;
    //         out.r = 0.;
    //         out.g = 1.;
    //     }
    //     return out;
    // }

    let layer = select_decor(i32(cell_x), i32(cell_y));
    if layer != -1 {
        let decor = textureSample(
            decor_array, decor_array_sampler,
            vec2(f32(i32(cell_size.x) - x) / f32(cell_size.x), (f32(i32(cell_size.y) - y)) / f32(cell_size.y)),
            layer
        );
        if decor.a > 0.1 {
            return decor;
        }
    }

    return textureSample(base_color_texture, base_color_sampler, vec2(f32(x) / f32(cell_size.x), f32(y) / f32(cell_size.y))) * vec4<f32>(0.09, 0.8, 0.2, 1.0);
}

@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    var out: VertexOutput;
    // out.position = vec4(vertex.position, 1.0);
    let have_width = view.viewport.z / 2.;
    let have_height = view.viewport.w / 2.;
    let cell_size = textureDimensions(base_color_texture);
    if vertex.vertex_index == 0 {
        out.position = vec4(-1., -1., 0.0, 1.0);
        out.uv = vec2(
            view.world_position.x - have_width,
            view.world_position.y - have_height,
        );
    } 
    else if vertex.vertex_index == 1 {
        out.position = vec4(1, -1, 0.0, 1.0);
                out.uv = vec2(
            view.world_position.x + have_width,
            view.world_position.y - have_height,
        );
    } else if vertex.vertex_index == 2 {
        out.position = vec4(-1, 1, 0.0, 1.0);
                out.uv = vec2(
            view.world_position.x - have_width,
            view.world_position.y + have_height,
        );
    } else {
        out.position = vec4(1, 1, 0.0, 1.0);
                out.uv = vec2(
            view.world_position.x + have_width,
            view.world_position.y + have_height,
        );
    }

    return out;
}

fn find_edge(x: i32, y: i32) -> f32 {
    let nx = f32(x) / 100.;
    let ny = f32(y) / 100.;
    let n = fbm(vec2<f32>(nx, ny));
    return n;
}

fn select_decor(x: i32, y: i32) -> i32 {
    let nx = f32(x) / 100.;
    let ny = f32(y) / 100.;
    var noise = fbm(vec2<f32>(nx, ny));
    let noise_up = fbm(vec2<f32>(nx, ny + 0.1));
    let noise_down = fbm(vec2<f32>(nx, ny - 0.1));
    let noise_left = fbm(vec2<f32>(nx - 0.1, ny));
    let noise_right = fbm(vec2<f32>(nx + 0.1, ny));
    noise = (noise + 1) / 2.; // Normalize to [0, 1]
    let j = x >> 31;
    let k = y >> 31;
    let t = ((y ^ k) + k);
    let i = ((((x ^ j) + j) * (-t) - 1) * 569) % 258161;
    let n = x + y * 2;

    let g_h = (noise_left - noise_right);
    let g_v = (noise_up - noise_down);
    var tg: f32 = 0.;
    if (n % 1024) > 512 {
        tg = (g_h - g_v) / 2.;
    } else {
        tg = (g_v - g_h) / 2.;
    };
    if tg > 0.01 && tg < 0.03 {
        // if noise 
        if i % 3 == 0 {
            return 7; // dirt
        }
        if i % 7 == 0 {
            return 8; // coarse dirt
        }
        if i % 13 == 0 {
            return 9; // rooted dirt
        }
        return 7; // dirt
    }


    // if i % 17 != 0 {
    //     return -1; // No decor selected, return -1
    // }

    noise = noise * 2 - 1; // Normalize to [-1, 1]
    {
        let n_up = fbm(vec2<f32>(nx, ny + 0.01));
        let n_down = fbm(vec2<f32>(nx, ny - 0.01));
        let n = fbm(vec2<f32>(nx, ny));
        let g_up = n_up - noise;
        let g_down = n_down - noise;
        if g_up < 0.001 && g_up > 0 && (x * 3 + y) % 21 == 0 {
            return 15;
        } else if g_down > -0.001 && g_down < 0 && (x * 3 + y) % 21 == 1 {
            return 14;
        }
    }

    if noise > noise_up && noise > noise_down { 
        if (i + x + y) % 13 == 0 {
            return 10 + (i % 4); // mushrooms
        } else if (i + x * 6 + y * 5) % 21 == 0 {
            return i % 14;
        } else {
            return -1; // brown mushroom
        }
    }
    if noise > noise_down && noise_left + noise_right > 0. {
        if i % 5 == 0 {
            if noise_down + noise_up > noise && noise_left + noise_right < noise {
                return 3;
            } else if noise_down + noise_left > noise && noise_up + noise_right < noise {
                return 4;
            } else if noise_up + noise_left > noise && noise_down + noise_right > noise && noise_down < 0.{
                return 5;
            } else if noise_down + noise_right > noise && noise_up + noise_left < noise {
                return 6;
            } else {
                return 3 + (i % 4); // tulips
            }
        } else {
            return -1;
        }
    }
    if noise > noise_left || noise > noise_right {
        if i % 17 == 0 {
            return (i + x + y) % 2; // flowers
        }
    }

    // 0 - 1 are flowers
    // 2 is wither rose
    // 3 - 6 are tulips
    // 7 - 9 are dirt variants
    // 10 - 13 are mushrooms
    // 14 - 15 are rose bushes

    if noise < noise_up && noise < noise_down && noise < noise_left && noise < noise_right && noise_down > noise_up && noise_left< noise_right && noise > 0.{
        return 2; // wither rose
    }
    return -1; // No decor selected, return -1
}


//  MIT License. © Ian McEwan, Stefan Gustavson, Munrocket, Johan Helsing
//
fn mod289(x: vec2<f32>) -> vec2<f32> {
    return x - floor(x * (1. / 289.)) * 289.;
}

fn mod289_3(x: vec3<f32>) -> vec3<f32> {
    return x - floor(x * (1. / 289.)) * 289.;
}

fn permute3(x: vec3<f32>) -> vec3<f32> {
    return mod289_3(((x * 34.) + 1.) * x);
}

//  MIT License. © Ian McEwan, Stefan Gustavson, Munrocket
fn simplexNoise2(v: vec2<f32>) -> f32 {
    let C = vec4<f32>(
        0.211324865405187, // (3.0-sqrt(3.0))/6.0
        0.366025403784439, // 0.5*(sqrt(3.0)-1.0)
        -0.577350269189626, // -1.0 + 2.0 * C.x
        0.024390243902439 // 1.0 / 41.0
    );

    // First corner
    var i = floor(v + dot(v, C.yy));
    let x0 = v - i + dot(i, C.xx);

    // Other corners
    var i1 = select(vec2(0., 1.), vec2(1., 0.), x0.x > x0.y);

    // x0 = x0 - 0.0 + 0.0 * C.xx ;
    // x1 = x0 - i1 + 1.0 * C.xx ;
    // x2 = x0 - 1.0 + 2.0 * C.xx ;
    var x12 = x0.xyxy + C.xxzz;
    x12.x = x12.x - i1.x;
    x12.y = x12.y - i1.y;

    // Permutations
    i = mod289(i); // Avoid truncation effects in permutation

    var p = permute3(permute3(i.y + vec3(0., i1.y, 1.)) + i.x + vec3(0., i1.x, 1.));
    var m = max(0.5 - vec3(dot(x0, x0), dot(x12.xy, x12.xy), dot(x12.zw, x12.zw)), vec3(0.));
    m *= m;
    m *= m;

    // Gradients: 41 points uniformly over a line, mapped onto a diamond.
    // The ring size 17*17 = 289 is close to a multiple of 41 (41*7 = 287)
    let x = 2. * fract(p * C.www) - 1.;
    let h = abs(x) - 0.5;
    let ox = floor(x + 0.5);
    let a0 = x - ox;

    // Normalize gradients implicitly by scaling m
    // Approximation of: m *= inversesqrt( a0*a0 + h*h );
    m *= 1.79284291400159 - 0.85373472095314 * (a0 * a0 + h * h);

    // Compute final noise value at P
    let g = vec3(a0.x * x0.x + h.x * x0.y, a0.yz * x12.xz + h.yz * x12.yw);
    return 130. * dot(m, g);
}

const m2: mat2x2<f32> = mat2x2<f32>(vec2(0.1, 0.15), vec2(-0.15, 0.1));

fn fbm(p: vec2<f32>) -> f32 {
    var f: f32 = -0.;
    var v = p;
    f = f + 0.5000 * simplexNoise2(v); v = m2 * v * 2.02;
    f = f + 0.2500 * simplexNoise2(v); v = m2 * v * 2.03;
    f = f + 0.250 * simplexNoise2(v); v = m2 * v * 2.01;
    f = f + 0.1625 * simplexNoise2(v);
    return f;
}