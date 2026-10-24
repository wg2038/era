/**
 * WebGPU WGSL 着色器：宣纸肌理 + 水墨流体扩散 + 年代情绪色调与微尘粒子
 */
export const INK_WASH_WGSL = /* wgsl */ `
struct Uniforms {
  resolution: vec2<f32>,
  time: f32,
  eraTone: f32, // 0.0: 1900s, 0.25: 1930s, 0.5: 1950s, 0.75: 1980s, 1.0: 2020s
};

@group(0) @binding(0) var<uniform> u: Uniforms;

// 伪随机哈希
fn hash21(p: vec2<f32>) -> f32 {
  var p3 = fract(vec3<f32>(p.xyx) * 0.1031);
  p3 += dot(p3, p3.yzx + 33.33);
  return fract((p3.x + p3.y) * p3.z);
}

// 2D 连续噪声
fn noise(p: vec2<f32>) -> f32 {
  let i = floor(p);
  let f = fract(p);
  let u_smooth = f * f * (3.0 - 2.0 * f);

  let a = hash21(i + vec2<f32>(0.0, 0.0));
  let b = hash21(i + vec2<f32>(1.0, 0.0));
  let c = hash21(i + vec2<f32>(0.0, 1.0));
  let d = hash21(i + vec2<f32>(1.0, 1.0));

  return mix(mix(a, b, u_smooth.x), mix(c, d, u_smooth.x), u_smooth.y);
}

// 分形布朗运动 (fBm)：模拟古宣纸帘纹与水墨多层渗晕
fn fbm(p: vec2<f32>) -> f32 {
  var v = 0.0;
  var a = 0.5;
  var shift = vec2<f32>(100.0, 100.0);
  var pos = p;
  for (var i = 0; i < 4; i = i + 1) {
    v += a * noise(pos);
    pos = pos * 2.0 + shift;
    a *= 0.5;
  }
  return v;
}

struct VertexOutput {
  @builtin(position) position: vec4<f32>,
  @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) vertex_id: u32) -> VertexOutput {
  var pos = array<vec2<f32>, 6>(
    vec2<f32>(-1.0, -1.0),
    vec2<f32>( 1.0, -1.0),
    vec2<f32>(-1.0,  1.0),
    vec2<f32>(-1.0,  1.0),
    vec2<f32>( 1.0, -1.0),
    vec2<f32>( 1.0,  1.0)
  );

  var out: VertexOutput;
  out.position = vec4<f32>(pos[vertex_id], 0.0, 1.0);
  out.uv = pos[vertex_id] * 0.5 + 0.5;
  return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
  let aspect = u.resolution.x / u.resolution.y;
  var uv = in.uv;
  uv.x *= aspect;

  let t = u.time * 0.05;

  // 1. 宣纸帘纹微观肌理基底
  let paperFiber = noise(uv * 120.0) * 0.04 + noise(uv * 40.0) * 0.02;
  let paperBase = vec3<f32>(0.93, 0.90, 0.84) - vec3<f32>(paperFiber);

  // 2. 模拟水墨流体缓慢浸润渗开 (Domain Warping)
  var q = vec2<f32>(fbm(uv + vec2<f32>(0.0, t)), fbm(uv + vec2<f32>(5.2, 1.3 - t)));
  var r = vec2<f32>(fbm(uv + 4.0 * q + vec2<f32>(1.7, 9.2)), fbm(uv + 4.0 * q + vec2<f32>(8.3, 2.8)));
  let inkSpread = fbm(uv + 3.0 * r);

  // 3. 依据年代参数过渡背景水墨与微粒色调
  // 1900-1920: 晚清古墨泛黄微褐
  let era0 = vec3<f32>(0.12, 0.10, 0.08); 
  // 1930-1940: 烽火灰烬深墨冷灰
  let era1 = vec3<f32>(0.08, 0.08, 0.09); 
  // 1950-1970: 工业赭红土黄暖调
  let era2 = vec3<f32>(0.15, 0.08, 0.07); 
  // 1980-1990: 改革开放海风淡青蓝
  let era3 = vec3<f32>(0.07, 0.11, 0.14); 
  // 2000-2025: 现代数码黛绿雅韵
  let era4 = vec3<f32>(0.06, 0.12, 0.10); 

  var inkColor = era0;
  if (u.eraTone < 0.25) {
    inkColor = mix(era0, era1, u.eraTone / 0.25);
  } else if (u.eraTone < 0.5) {
    inkColor = mix(era1, era2, (u.eraTone - 0.25) / 0.25);
  } else if (u.eraTone < 0.75) {
    inkColor = mix(era2, era3, (u.eraTone - 0.50) / 0.25);
  } else {
    inkColor = mix(era3, era4, (u.eraTone - 0.75) / 0.25);
  }

  // 混合宣纸底色与流体水墨晕染
  let inkFactor = smoothstep(0.25, 0.85, inkSpread) * 0.45;
  var finalColor = mix(paperBase, inkColor, inkFactor);

  // 4. 浮游墨尘微粒粒子
  let particleNoise = hash21(floor(uv * 18.0 + vec2<f32>(0.0, u.time * 0.1)));
  if (particleNoise > 0.985) {
    finalColor += vec3<f32>(0.04);
  }

  return vec4<f32>(finalColor, 1.0);
}
`;
