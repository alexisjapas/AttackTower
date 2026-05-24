use bevy::prelude::*;
use std::path::PathBuf;

use crate::common::*;

// ────────────────────────────────────────────────────────────────────────────
// Parameters
// ────────────────────────────────────────────────────────────────────────────

/// Identifier for every graphics/video parameter that can be tweaked. Using a
/// named enum (instead of bare indices) keeps the menu layout (positions per
/// tab) decoupled from label/description lookup.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ParamId {
    // Video / display
    Fullscreen,
    VSync,
    Msaa,
    Hdr,
    Exposure, // sub-param of Hdr
    Tonemapping,
    // Graphics / quality
    Raytracing,
    Dlss,
    DlssQuality, // sub-param of Dlss
    Taa,
    Fxaa,
    Bloom,
    BloomIntensity, // sub-param of Bloom
    Atmosphere,
    VolumetricFog,
    FogDensity, // sub-param of VolumetricFog
    DistanceFog,
    Ssao,
    SsaoQuality, // sub-param of Ssao
    Shadows,
    MotionBlur,
}

/// A row in the settings menu. The `Preset` slot only exists on the Graphics
/// tab; `Back` closes the menu.
#[derive(Clone, Copy)]
pub enum MenuSlot {
    Preset,
    Param(ParamId),
    Back,
}

/// Build the menu rows for `tab` given the current settings. Sub-parameters
/// only appear when their parent toggle is on, so the slot list shrinks /
/// grows as the user enables features.
pub fn tab_slots(tab: SettingsTab, s: &GameSettings) -> Vec<MenuSlot> {
    let mut v = Vec::with_capacity(16);
    match tab {
        SettingsTab::Video => {
            v.push(MenuSlot::Param(ParamId::Fullscreen));
            v.push(MenuSlot::Param(ParamId::VSync));
            v.push(MenuSlot::Param(ParamId::Msaa));
            v.push(MenuSlot::Param(ParamId::Hdr));
            if s.hdr {
                v.push(MenuSlot::Param(ParamId::Exposure));
            }
            v.push(MenuSlot::Param(ParamId::Tonemapping));
        }
        SettingsTab::Graphics => {
            v.push(MenuSlot::Preset);
            v.push(MenuSlot::Param(ParamId::Raytracing));
            v.push(MenuSlot::Param(ParamId::Dlss));
            if s.dlss {
                v.push(MenuSlot::Param(ParamId::DlssQuality));
            }
            v.push(MenuSlot::Param(ParamId::Taa));
            v.push(MenuSlot::Param(ParamId::Fxaa));
            v.push(MenuSlot::Param(ParamId::Bloom));
            if s.bloom {
                v.push(MenuSlot::Param(ParamId::BloomIntensity));
            }
            v.push(MenuSlot::Param(ParamId::Atmosphere));
            v.push(MenuSlot::Param(ParamId::VolumetricFog));
            if s.volumetric_fog {
                v.push(MenuSlot::Param(ParamId::FogDensity));
            }
            v.push(MenuSlot::Param(ParamId::DistanceFog));
            v.push(MenuSlot::Param(ParamId::Ssao));
            if s.ssao {
                v.push(MenuSlot::Param(ParamId::SsaoQuality));
            }
            v.push(MenuSlot::Param(ParamId::Shadows));
            v.push(MenuSlot::Param(ParamId::MotionBlur));
        }
    }
    v.push(MenuSlot::Back);
    v
}

pub fn slot_count(tab: SettingsTab, s: &GameSettings) -> usize {
    tab_slots(tab, s).len()
}

// ────────────────────────────────────────────────────────────────────────────
// Preset
// ────────────────────────────────────────────────────────────────────────────

#[derive(Resource, Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum GraphicsPreset {
    Low,
    Medium,
    High,
    Ultra,
    #[default]
    Custom,
}

impl GraphicsPreset {
    pub fn label(self) -> &'static str {
        match self {
            GraphicsPreset::Low => "Low",
            GraphicsPreset::Medium => "Medium",
            GraphicsPreset::High => "High",
            GraphicsPreset::Ultra => "Ultra",
            GraphicsPreset::Custom => "Custom",
        }
    }

    /// Cycle through the four named presets. `Custom` is never produced by
    /// activation — it is derived by `detect` when no preset matches.
    pub fn cycle(self) -> Self {
        match self {
            GraphicsPreset::Low => GraphicsPreset::Medium,
            GraphicsPreset::Medium => GraphicsPreset::High,
            GraphicsPreset::High => GraphicsPreset::Ultra,
            GraphicsPreset::Ultra | GraphicsPreset::Custom => GraphicsPreset::Low,
        }
    }

    /// Apply this preset's quality settings to `s`. Display/video fields
    /// (fullscreen, vsync, hdr, msaa, tonemapping) and sub-parameters
    /// (exposure, bloom intensity, dlss quality, ...) are left untouched —
    /// they are managed independently from the preset.
    pub fn apply(self, s: &mut GameSettings, dlss_supported: bool) {
        match self {
            GraphicsPreset::Low => {
                s.raytracing = false;
                s.dlss = false;
                s.taa = false;
                s.fxaa = true; // cheap fallback AA
                s.bloom = false;
                s.atmosphere = false;
                s.volumetric_fog = false;
                s.distance_fog = false;
                s.ssao = false;
                s.shadows = false;
                s.motion_blur = false;
            }
            GraphicsPreset::Medium => {
                s.raytracing = false;
                s.dlss = false;
                s.taa = false;
                s.fxaa = true;
                s.bloom = true;
                s.atmosphere = false;
                s.volumetric_fog = false;
                s.distance_fog = true;
                s.ssao = false;
                s.shadows = true;
                s.motion_blur = false;
            }
            GraphicsPreset::High => {
                s.raytracing = false;
                s.dlss = false;
                s.taa = true;
                s.fxaa = false;
                s.bloom = true;
                s.atmosphere = true;
                s.volumetric_fog = false;
                s.distance_fog = true;
                s.ssao = true;
                s.shadows = true;
                s.motion_blur = false;
            }
            GraphicsPreset::Ultra => {
                s.raytracing = cfg!(feature = "raytracing");
                s.dlss = cfg!(feature = "dlss") && dlss_supported;
                s.taa = true;
                s.fxaa = false;
                s.bloom = true;
                s.atmosphere = true;
                s.volumetric_fog = true;
                s.distance_fog = true;
                s.ssao = true;
                s.shadows = true;
                s.motion_blur = true;
            }
            GraphicsPreset::Custom => {}
        }
    }

    pub fn detect(s: &GameSettings, dlss_supported: bool) -> Self {
        for preset in [Self::Low, Self::Medium, Self::High, Self::Ultra] {
            let mut probe = *s;
            preset.apply(&mut probe, dlss_supported);
            if quality_matches(s, &probe) {
                return preset;
            }
        }
        Self::Custom
    }
}

fn quality_matches(a: &GameSettings, b: &GameSettings) -> bool {
    a.raytracing == b.raytracing
        && a.dlss == b.dlss
        && a.taa == b.taa
        && a.fxaa == b.fxaa
        && a.bloom == b.bloom
        && a.atmosphere == b.atmosphere
        && a.volumetric_fog == b.volumetric_fog
        && a.distance_fog == b.distance_fog
        && a.ssao == b.ssao
        && a.shadows == b.shadows
        && a.motion_blur == b.motion_blur
}

pub fn update_graphics_preset(
    settings: Res<GameSettings>,
    dlss_avail: Res<DlssAvailable>,
    mut preset: ResMut<GraphicsPreset>,
) {
    if !settings.is_changed() && !dlss_avail.is_changed() {
        return;
    }
    let new = GraphicsPreset::detect(&settings, dlss_avail.0);
    if *preset != new {
        *preset = new;
    }
}

// ────────────────────────────────────────────────────────────────────────────
// Per-parameter descriptions
// ────────────────────────────────────────────────────────────────────────────

#[derive(Clone, Copy)]
pub enum Impact {
    /// No measurable cost on this resource.
    None,
    Low,
    Medium,
    High,
}

impl Impact {
    pub fn label(self) -> &'static str {
        match self {
            Impact::None => "none",
            Impact::Low => "low",
            Impact::Medium => "medium",
            Impact::High => "high",
        }
    }

    pub fn color(self) -> Color {
        match self {
            Impact::None => Color::srgb(0.55, 0.60, 0.70),
            Impact::Low => Color::srgb(0.55, 0.85, 0.55),
            Impact::Medium => Color::srgb(0.95, 0.85, 0.45),
            Impact::High => Color::srgb(0.95, 0.55, 0.45),
        }
    }
}

pub struct ParamDescription {
    pub title: &'static str,
    pub functional: &'static str,
    pub technical: &'static str,
    pub cpu: Impact,
    pub gpu: Impact,
    pub ram: Impact,
    pub vram: Impact,
}

pub enum DescriptionKind {
    Param(ParamDescription),
    Preset {
        title: &'static str,
        functional: &'static str,
        technical: &'static str,
    },
    None,
}

/// Description block for the menu slot at `menu_index` within `tab`.
pub fn description_for(
    tab: SettingsTab,
    menu_index: usize,
    preset: GraphicsPreset,
    settings: &GameSettings,
) -> DescriptionKind {
    let slots = tab_slots(tab, settings);
    let Some(slot) = slots.get(menu_index) else {
        return DescriptionKind::None;
    };
    match slot {
        MenuSlot::Preset => DescriptionKind::Preset {
            title: "Graphics preset",
            functional: concat!(
                "Apply a coherent quality bundle in one click.\n",
                "Low: everything off, max FPS. Medium: comfortable visuals. ",
                "High: rich visuals. Ultra: every effect on (RT + DLSS when available).\n",
                "Switches to Custom as soon as any graphics parameter diverges from the chosen preset."
            ),
            technical: match preset {
                GraphicsPreset::Low => "Active: Low. No optional effect, cheapest render path.",
                GraphicsPreset::Medium => "Active: Medium. Bloom + distance fog enabled.",
                GraphicsPreset::High => "Active: High. TAA + atmosphere + bloom + distance fog.",
                GraphicsPreset::Ultra => {
                    "Active: Ultra. Raytracing + DLSS (if supported), volumetric fog and full post-process."
                }
                GraphicsPreset::Custom => {
                    "Active: Custom (current values do not match any preset)."
                }
            },
        },
        MenuSlot::Param(id) => DescriptionKind::Param(param_description(*id)),
        MenuSlot::Back => DescriptionKind::None,
    }
}

pub fn param_description(id: ParamId) -> ParamDescription {
    match id {
        ParamId::Fullscreen => ParamDescription {
            title: "Fullscreen",
            functional: concat!(
                "Switches between windowed and borderless fullscreen.\n",
                "Fullscreen covers the whole display with no window decorations."
            ),
            technical: concat!(
                "Bevy swaps WindowMode between Windowed and BorderlessFullscreen.\n",
                "The compositor can fast-path frames, often reducing input latency."
            ),
            cpu: Impact::None,
            gpu: Impact::None,
            ram: Impact::None,
            vram: Impact::None,
        },
        ParamId::VSync => ParamDescription {
            title: "VSync",
            functional: concat!(
                "Synchronises frame presentation with the monitor refresh rate.\n",
                "Eliminates tearing at the cost of mild input lag and a refresh-rate frame cap."
            ),
            technical: concat!(
                "Switches PresentMode between AutoVsync (FIFO) and AutoNoVsync (Mailbox/Immediate).\n",
                "Frames wait for the next VBLANK before being presented to the swapchain."
            ),
            cpu: Impact::None,
            gpu: Impact::None,
            ram: Impact::None,
            vram: Impact::None,
        },
        ParamId::Hdr => ParamDescription {
            title: "HDR",
            functional: concat!(
                "Renders the scene to a high-dynamic-range buffer for better\n",
                "highlight preservation and richer bloom/tonemapping fidelity."
            ),
            technical: concat!(
                "Toggles Camera::hdr, switching the main render target from\n",
                "Rgba8 (SDR) to Rgba16Float (HDR). Required for proper bloom."
            ),
            cpu: Impact::None,
            gpu: Impact::Low,
            ram: Impact::None,
            vram: Impact::Low,
        },
        ParamId::Tonemapping => ParamDescription {
            title: "Tonemapping",
            functional: concat!(
                "Maps the HDR render to displayable SDR colours.\n",
                "ACES = cinematic, Tony McMapface = neutral, Reinhard = simple, None = raw HDR."
            ),
            technical: concat!(
                "3D LUT or analytic curve applied as a post-process pass,\n",
                "after bloom and before swapchain present."
            ),
            cpu: Impact::None,
            gpu: Impact::None,
            ram: Impact::None,
            vram: Impact::None,
        },
        ParamId::Raytracing => ParamDescription {
            title: "Raytracing (Solari)",
            functional: concat!(
                "Global illumination, shadows and reflections evaluated by ray tracing.\n",
                "Indirect light bounces and contact shadows look physically accurate."
            ),
            technical: concat!(
                "Bevy Solari: ReSTIR DI/GI on GPU, requires an RTX-class card.\n",
                "Builds a BVH (RaytracingMesh3d) and forces the deferred + prepass pipeline."
            ),
            cpu: Impact::Low,
            gpu: Impact::High,
            ram: Impact::Medium,
            vram: Impact::High,
        },
        ParamId::Dlss => ParamDescription {
            title: "DLSS",
            functional: concat!(
                "AI upscaler: renders the scene at a lower internal resolution then\n",
                "reconstructs the native-resolution image, usually a big FPS win."
            ),
            technical: concat!(
                "NVIDIA NGX Ray Reconstruction on RTX GPUs.\n",
                "Requires motion vectors + depth prepass; runs on the Tensor cores."
            ),
            cpu: Impact::None,
            // Net GPU cost is negative (DLSS saves more than it adds), so we
            // report Low to convey "extra work is small".
            gpu: Impact::Low,
            ram: Impact::None,
            vram: Impact::Medium,
        },
        ParamId::Taa => ParamDescription {
            title: "TAA",
            functional: concat!(
                "Temporal anti-aliasing: smooths edges by accumulating several\n",
                "subpixel-jittered frames over time."
            ),
            technical: concat!(
                "Reprojection via motion vectors + history buffer in VRAM.\n",
                "Requires MotionVectorPrepass and a jittered camera; MSAA must stay off."
            ),
            cpu: Impact::None,
            gpu: Impact::Medium,
            ram: Impact::None,
            vram: Impact::Medium,
        },
        ParamId::Bloom => ParamDescription {
            title: "Bloom",
            functional: concat!(
                "Soft halo around very bright pixels (sun, flames).\n",
                "Sells the impression of HDR luminance and contrast."
            ),
            technical: concat!(
                "Mip down/up chain + additive blend in post-process.\n",
                "Reads the HDR framebuffer; cost scales with screen resolution."
            ),
            cpu: Impact::None,
            gpu: Impact::Low,
            ram: Impact::None,
            vram: Impact::None,
        },
        ParamId::Atmosphere => ParamDescription {
            title: "Atmosphere",
            functional: concat!(
                "Physical atmospheric scattering: sky colour, distant haze\n",
                "and sun colour that follows its elevation angle."
            ),
            technical: concat!(
                "Pre-computed LUTs (transmittance, sky-view, multiscattering)\n",
                "plus Rayleigh + Mie integral over scene depth."
            ),
            cpu: Impact::None,
            gpu: Impact::Medium,
            ram: Impact::None,
            vram: Impact::Medium,
        },
        ParamId::VolumetricFog => ParamDescription {
            title: "Volumetric fog",
            functional: concat!(
                "Fog volumes crossed by the directional light produce visible\n",
                "rays of light (god rays)."
            ),
            technical: concat!(
                "Raymarching in a 3D froxel grid (frustum voxels).\n",
                "Requires VolumetricLight on the sun and a FogVolume in the scene."
            ),
            cpu: Impact::None,
            gpu: Impact::High,
            ram: Impact::None,
            vram: Impact::Medium,
        },
        ParamId::DistanceFog => ParamDescription {
            title: "Distance fog",
            functional: concat!(
                "Fades far-away objects into a fog tint that grows\n",
                "with camera distance."
            ),
            technical: concat!(
                "Exponential-squared mix evaluated in the fragment shader.\n",
                "Adds a couple of ALU ops per pixel, no extra buffer."
            ),
            cpu: Impact::None,
            gpu: Impact::None,
            ram: Impact::None,
            vram: Impact::None,
        },
        ParamId::Msaa => ParamDescription {
            title: "MSAA",
            functional: concat!(
                "Multi-sample anti-aliasing: super-samples geometry edges\n",
                "using 2/4/8 hardware samples per pixel.",
                "\nIgnored when Raytracing or TAA force the deferred pipeline."
            ),
            technical: concat!(
                "GPU-resolved multisampled render target.\n",
                "Bandwidth + memory scale linearly with the sample count."
            ),
            cpu: Impact::None,
            gpu: Impact::Medium,
            ram: Impact::None,
            vram: Impact::Medium,
        },
        ParamId::Fxaa => ParamDescription {
            title: "FXAA",
            functional: concat!(
                "Fast approximate anti-aliasing: a cheap post-process pass\n",
                "that softens jagged edges from luma contrast."
            ),
            technical: concat!(
                "Single fullscreen pass over the LDR/HDR colour buffer.\n",
                "Trades some sharpness for an O(1) screen-space cost."
            ),
            cpu: Impact::None,
            gpu: Impact::Low,
            ram: Impact::None,
            vram: Impact::None,
        },
        ParamId::Ssao => ParamDescription {
            title: "SSAO",
            functional: concat!(
                "Screen-space ambient occlusion: darkens crevices and contact\n",
                "points where indirect light would be blocked."
            ),
            technical: concat!(
                "GTAO-style sampling in screen space. Requires depth +\n",
                "normal prepasses; cost scales with the quality preset."
            ),
            cpu: Impact::None,
            gpu: Impact::Medium,
            ram: Impact::None,
            vram: Impact::Low,
        },
        ParamId::Shadows => ParamDescription {
            title: "Shadows",
            functional: concat!(
                "Cascaded shadow maps cast by the sun.\n",
                "Off means flat lighting with no occlusion from the sun."
            ),
            technical: concat!(
                "Multiple shadow cascades rendered each frame and sampled\n",
                "in the PBR shader. Cost scales with cascade resolution + count."
            ),
            cpu: Impact::Low,
            gpu: Impact::Medium,
            ram: Impact::None,
            vram: Impact::Medium,
        },
        ParamId::MotionBlur => ParamDescription {
            title: "Motion blur",
            functional: concat!(
                "Smears moving objects in the direction of their motion,\n",
                "selling speed and reducing judder at low frame rates."
            ),
            technical: concat!(
                "Per-pixel directional blur driven by motion vectors.\n",
                "Requires Depth + MotionVector prepasses."
            ),
            cpu: Impact::None,
            gpu: Impact::Low,
            ram: Impact::None,
            vram: Impact::Low,
        },
        ParamId::Exposure => ParamDescription {
            title: "Exposure",
            functional: concat!(
                "Selects how bright the HDR scene appears after tonemapping.\n",
                "Cycles Low / Default / High (EV100 11 / 13 / 15)."
            ),
            technical: concat!(
                "Updates the Exposure component (ev100) on the camera.\n",
                "Acts as a multiplier on linear light before tonemapping."
            ),
            cpu: Impact::None,
            gpu: Impact::None,
            ram: Impact::None,
            vram: Impact::None,
        },
        ParamId::BloomIntensity => ParamDescription {
            title: "Bloom intensity",
            functional: concat!(
                "Controls the visible amount of bloom glow.\n",
                "Cycles Low / Default / High."
            ),
            technical: concat!(
                "Scales the `intensity` field of the Bloom post-process.\n",
                "Same shader cost regardless of intensity."
            ),
            cpu: Impact::None,
            gpu: Impact::None,
            ram: Impact::None,
            vram: Impact::None,
        },
        ParamId::DlssQuality => ParamDescription {
            title: "DLSS quality",
            functional: concat!(
                "DLSS render preset: trades internal resolution for FPS.\n",
                "Performance < Balanced < Quality < DLAA < Auto (driver-picked)."
            ),
            technical: concat!(
                "Maps to DlssPerfQualityMode. Lower quality renders the scene\n",
                "at a smaller internal resolution then upscales with the NN."
            ),
            cpu: Impact::None,
            gpu: Impact::Low,
            ram: Impact::None,
            vram: Impact::None,
        },
        ParamId::SsaoQuality => ParamDescription {
            title: "SSAO quality",
            functional: concat!(
                "Number of samples used by SSAO per pixel.\n",
                "Cycles Low / Medium / High / Ultra (4 / 8 / 18 / 54 samples)."
            ),
            technical: concat!(
                "Maps to ScreenSpaceAmbientOcclusionQualityLevel.\n",
                "Cost grows roughly linearly with sample count."
            ),
            cpu: Impact::None,
            gpu: Impact::Medium,
            ram: Impact::None,
            vram: Impact::None,
        },
        ParamId::FogDensity => ParamDescription {
            title: "Fog density",
            functional: concat!(
                "Thickness of the volumetric fog volume.\n",
                "Higher density = more visible god rays, more atmospheric depth."
            ),
            technical: concat!(
                "Scales the ambient intensity / scattering density inside the\n",
                "VolumetricFog component. Cost stays the same."
            ),
            cpu: Impact::None,
            gpu: Impact::None,
            ram: Impact::None,
            vram: Impact::None,
        },
    }
}

/// Resolve a short human-readable label for a single parameter, suitable as
/// the button text. Sub-parameters use a `   - ` indent prefix so the
/// hierarchy reads at a glance.
pub fn param_label(id: ParamId, s: &GameSettings, dlss_supported: bool) -> String {
    match id {
        ParamId::Fullscreen => format!("Fullscreen: {}", on_off(s.fullscreen)),
        ParamId::VSync => format!("VSync: {}", on_off(s.vsync)),
        ParamId::Msaa => format!("MSAA: {}", msaa_label(s.msaa)),
        ParamId::Hdr => format!("HDR: {}", on_off(s.hdr)),
        ParamId::Exposure => format!("  - Exposure: {}", exposure_label(s.exposure)),
        ParamId::Tonemapping => format!("Tonemapping: {}", tonemapping_label(s.tonemapping)),
        ParamId::Raytracing => {
            if cfg!(feature = "raytracing") {
                format!("Raytracing (Solari): {}", on_off(s.raytracing))
            } else {
                "Raytracing (Solari): N/A".into()
            }
        }
        ParamId::Dlss => {
            if cfg!(feature = "dlss") && dlss_supported {
                format!("DLSS: {}", on_off(s.dlss))
            } else {
                "DLSS: N/A".into()
            }
        }
        ParamId::DlssQuality => format!("  - DLSS quality: {}", dlss_quality_label(s.dlss_quality)),
        ParamId::Taa => format!("TAA: {}", on_off(s.taa)),
        ParamId::Fxaa => format!("FXAA: {}", on_off(s.fxaa)),
        ParamId::Bloom => format!("Bloom: {}", on_off(s.bloom)),
        ParamId::BloomIntensity => {
            format!("  - Bloom intensity: {}", level_label(s.bloom_intensity))
        }
        ParamId::Atmosphere => format!("Atmosphere: {}", on_off(s.atmosphere)),
        ParamId::VolumetricFog => format!("Volumetric fog: {}", on_off(s.volumetric_fog)),
        ParamId::FogDensity => format!("  - Fog density: {}", level_label(s.fog_density)),
        ParamId::DistanceFog => format!("Distance fog: {}", on_off(s.distance_fog)),
        ParamId::Ssao => format!("SSAO: {}", on_off(s.ssao)),
        ParamId::SsaoQuality => format!("  - SSAO quality: {}", ssao_quality_label(s.ssao_quality)),
        ParamId::Shadows => format!("Shadows: {}", on_off(s.shadows)),
        ParamId::MotionBlur => format!("Motion blur: {}", on_off(s.motion_blur)),
    }
}

fn on_off(on: bool) -> &'static str {
    if on { "ON" } else { "OFF" }
}

pub fn tonemapping_label(idx: u8) -> &'static str {
    match idx {
        0 => "ACES Fitted",
        1 => "Tony McMapface",
        2 => "Reinhard",
        _ => "None",
    }
}

pub fn msaa_label(idx: u8) -> &'static str {
    match idx {
        2 => "2x",
        4 => "4x",
        8 => "8x",
        _ => "OFF",
    }
}

pub fn exposure_label(idx: u8) -> &'static str {
    match idx {
        0 => "Low",
        2 => "High",
        _ => "Default",
    }
}

/// EV100 value matching `exposure_label`.
pub fn exposure_ev100(idx: u8) -> f32 {
    match idx {
        0 => 11.0,
        2 => 15.0,
        _ => 13.0,
    }
}

pub fn level_label(idx: u8) -> &'static str {
    match idx {
        0 => "Low",
        2 => "High",
        _ => "Default",
    }
}

/// Bloom intensity multiplier matching `level_label`.
pub fn bloom_intensity_value(idx: u8) -> f32 {
    match idx {
        0 => 0.06,
        2 => 0.30,
        _ => 0.15, // Bloom::NATURAL default
    }
}

/// Volumetric-fog ambient intensity matching `level_label`.
pub fn fog_density_value(idx: u8) -> f32 {
    match idx {
        0 => 0.02,
        2 => 0.12,
        _ => 0.05,
    }
}

pub fn dlss_quality_label(idx: u8) -> &'static str {
    match idx {
        0 => "Performance",
        1 => "Balanced",
        2 => "Quality",
        3 => "DLAA",
        _ => "Auto",
    }
}

pub fn ssao_quality_label(idx: u8) -> &'static str {
    match idx {
        0 => "Low",
        1 => "Medium",
        3 => "Ultra",
        _ => "High",
    }
}

// ────────────────────────────────────────────────────────────────────────────
// Persistence
// ────────────────────────────────────────────────────────────────────────────

fn settings_path() -> Option<PathBuf> {
    let dir = if let Some(xdg) = std::env::var_os("XDG_CONFIG_HOME") {
        PathBuf::from(xdg)
    } else if let Some(home) = std::env::var_os("HOME") {
        PathBuf::from(home).join(".config")
    } else {
        return None;
    };
    Some(dir.join("attack_tower").join("settings.cfg"))
}

pub fn load_settings() -> GameSettings {
    let mut s = GameSettings::default();
    let Some(path) = settings_path() else {
        return s;
    };
    let Ok(text) = std::fs::read_to_string(&path) else {
        return s;
    };
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((k, v)) = line.split_once('=') else {
            continue;
        };
        let k = k.trim();
        let v = v.trim();
        match k {
            "fullscreen" => s.fullscreen = parse_bool(v).unwrap_or(s.fullscreen),
            "vsync" => s.vsync = parse_bool(v).unwrap_or(s.vsync),
            "hdr" => s.hdr = parse_bool(v).unwrap_or(s.hdr),
            "msaa" => s.msaa = v.parse().unwrap_or(s.msaa),
            "tonemapping" => s.tonemapping = v.parse().unwrap_or(s.tonemapping),
            "raytracing" => s.raytracing = parse_bool(v).unwrap_or(s.raytracing),
            "dlss" => s.dlss = parse_bool(v).unwrap_or(s.dlss),
            "taa" => s.taa = parse_bool(v).unwrap_or(s.taa),
            "fxaa" => s.fxaa = parse_bool(v).unwrap_or(s.fxaa),
            "bloom" => s.bloom = parse_bool(v).unwrap_or(s.bloom),
            "atmosphere" => s.atmosphere = parse_bool(v).unwrap_or(s.atmosphere),
            "volumetric_fog" => s.volumetric_fog = parse_bool(v).unwrap_or(s.volumetric_fog),
            "distance_fog" => s.distance_fog = parse_bool(v).unwrap_or(s.distance_fog),
            "ssao" => s.ssao = parse_bool(v).unwrap_or(s.ssao),
            "shadows" => s.shadows = parse_bool(v).unwrap_or(s.shadows),
            "motion_blur" => s.motion_blur = parse_bool(v).unwrap_or(s.motion_blur),
            "exposure" => s.exposure = v.parse().unwrap_or(s.exposure),
            "bloom_intensity" => s.bloom_intensity = v.parse().unwrap_or(s.bloom_intensity),
            "dlss_quality" => s.dlss_quality = v.parse().unwrap_or(s.dlss_quality),
            "ssao_quality" => s.ssao_quality = v.parse().unwrap_or(s.ssao_quality),
            "fog_density" => s.fog_density = v.parse().unwrap_or(s.fog_density),
            _ => {}
        }
    }
    if !cfg!(feature = "raytracing") {
        s.raytracing = false;
    }
    if !cfg!(feature = "dlss") {
        s.dlss = false;
    }
    if s.tonemapping > 3 {
        s.tonemapping = 0;
    }
    if !matches!(s.msaa, 0 | 2 | 4 | 8) {
        s.msaa = 0;
    }
    if s.exposure > 2 {
        s.exposure = 1;
    }
    if s.bloom_intensity > 2 {
        s.bloom_intensity = 1;
    }
    if s.dlss_quality > 4 {
        s.dlss_quality = 4;
    }
    if s.ssao_quality > 3 {
        s.ssao_quality = 2;
    }
    if s.fog_density > 2 {
        s.fog_density = 1;
    }
    s
}

pub fn save_settings(s: &GameSettings) {
    let Some(path) = settings_path() else {
        return;
    };
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let text = format!(
        "fullscreen = {}\nvsync = {}\nhdr = {}\nmsaa = {}\ntonemapping = {}\n\
         raytracing = {}\ndlss = {}\ntaa = {}\nfxaa = {}\nbloom = {}\n\
         atmosphere = {}\nvolumetric_fog = {}\ndistance_fog = {}\nssao = {}\n\
         shadows = {}\nmotion_blur = {}\n\
         exposure = {}\nbloom_intensity = {}\ndlss_quality = {}\nssao_quality = {}\nfog_density = {}\n",
        s.fullscreen,
        s.vsync,
        s.hdr,
        s.msaa,
        s.tonemapping,
        s.raytracing,
        s.dlss,
        s.taa,
        s.fxaa,
        s.bloom,
        s.atmosphere,
        s.volumetric_fog,
        s.distance_fog,
        s.ssao,
        s.shadows,
        s.motion_blur,
        s.exposure,
        s.bloom_intensity,
        s.dlss_quality,
        s.ssao_quality,
        s.fog_density,
    );
    let _ = std::fs::write(&path, text);
}

fn parse_bool(v: &str) -> Option<bool> {
    match v {
        "true" | "1" | "on" | "yes" => Some(true),
        "false" | "0" | "off" | "no" => Some(false),
        _ => None,
    }
}

pub fn persist_settings(settings: Res<GameSettings>, mut started: Local<bool>) {
    if !*started {
        *started = true;
        return;
    }
    if !settings.is_changed() {
        return;
    }
    save_settings(&settings);
}
