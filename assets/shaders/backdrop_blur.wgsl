// Frosted-glass UI material: disk-blur the scene behind the node, tint it, and
// clip to the node's rounded rect. See src/backdrop.rs for the render plumbing.
#import bevy_ui::ui_vertex_output::UiVertexOutput
#import bevy_render::view::View

// bind group 0 is always the UI view (binding 0) + globals (binding 1); we only
// need the view to turn the fragment's framebuffer position into a screen uv.
@group(0) @binding(0) var<uniform> view: View;

struct BackdropSettings {
    tint: vec4<f32>,
    blur_radius: f32,
}

@group(1) @binding(0) var<uniform> settings: BackdropSettings;
@group(1) @binding(1) var backdrop_texture: texture_2d<f32>;
@group(1) @binding(2) var backdrop_sampler: sampler;

const TAPS: i32 = 16;
const GOLDEN_ANGLE: f32 = 2.3999632;

// Signed distance to a rounded rectangle centered at the origin. `radius`
// follows Bevy's border-radius order: top-left, top-right, bottom-right,
// bottom-left.
fn rounded_rect_sd(p: vec2<f32>, half_size: vec2<f32>, radius: vec4<f32>) -> f32 {
    let left = p.x < 0.0;
    let top = p.y < 0.0;
    var r: f32;
    if (left && top) {
        r = radius.x;
    } else if (!left && top) {
        r = radius.y;
    } else if (!left && !top) {
        r = radius.z;
    } else {
        r = radius.w;
    }
    let q = abs(p) - half_size + vec2<f32>(r);
    return min(max(q.x, q.y), 0.0) + length(max(q, vec2<f32>(0.0))) - r;
}

@fragment
fn fragment(in: UiVertexOutput) -> @location(0) vec4<f32> {
    let screen_uv = (in.position.xy - view.viewport.xy) / view.viewport.zw;
    let texel = 1.0 / view.viewport.zw;

    // Golden-angle disk blur of the backdrop copy.
    var acc = vec3<f32>(0.0);
    for (var i = 0; i < TAPS; i = i + 1) {
        let t = (f32(i) + 0.5) / f32(TAPS);
        let radius = sqrt(t) * settings.blur_radius;
        let angle = f32(i) * GOLDEN_ANGLE;
        let offset = vec2<f32>(cos(angle), sin(angle)) * radius * texel;
        acc = acc + textureSampleLevel(backdrop_texture, backdrop_sampler, screen_uv + offset, 0.0).rgb;
    }
    let glass = mix(acc / f32(TAPS), settings.tint.rgb, settings.tint.a);

    // Rounded-rect coverage with a ~1px antialiased edge (dist is in pixels).
    let p = (in.uv - vec2<f32>(0.5)) * in.size;
    let dist = rounded_rect_sd(p, in.size * 0.5, in.border_radius);
    let coverage = clamp(0.5 - dist, 0.0, 1.0);

    return vec4<f32>(glass, coverage);
}
