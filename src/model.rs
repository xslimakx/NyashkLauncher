//! Plugin registry, settings, installed-version records and version helpers.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Category {
    Instrument,
    Effect,
    Tool,
    Analysis,
}

impl Category {
    pub const ALL: [Category; 4] = [Category::Instrument, Category::Effect, Category::Tool, Category::Analysis];
    pub fn label(self) -> &'static str {
        match self {
            Category::Instrument => "Instruments",
            Category::Effect => "Effects",
            Category::Tool => "Vocal tools",
            Category::Analysis => "Analysis",
        }
    }
}

pub const LAUNCHER_REPO: &str = "xslimakx/NyashkLauncher";

#[derive(Clone)]
pub struct Shot {
    pub caption: &'static str,
    pub data: &'static [u8],
}

#[derive(Clone)]
pub struct PluginDef {
    pub id: &'static str,
    pub name: &'static str,
    pub repo: &'static str,
    pub bundle: &'static str,
    pub category: Category,
    pub tagline: &'static str,
    pub description: &'static str,
    pub tags: &'static [&'static str],
    pub long: &'static str,
    pub features: &'static [&'static str],
    pub shots: &'static [Shot],
}

/// Built-in fallback list. The live list is `plugins()`: this plus `plugins.json` from the launcher repo.
pub const BUILTIN: &[PluginDef] = &[
    PluginDef {
        id: "synth",
        name: "NyashkSynth",
        repo: "xslimakx/NyashkSynth",
        bundle: "NyashkSynth.vst3",
        category: Category::Instrument,
        tagline: "Wavetable synthesizer",
        description: "Serum-style wavetable synth: 3 oscillators with wavetable, sample, granular, \
                      multisample and spectral modes, two filters, mod matrix, effect rack and 146 factory presets.",
        tags: &["VST3", "Synth", "Wavetable"],
        long: "A wavetable synthesizer in the monochrome Nyashk design. Three oscillators, each a wavetable, sample, granular, multisample or spectral source, run through two filters, a 16-slot mod matrix and a two-bus effect rack. Includes a wavetable editor, 146 factory presets and a powerful arpeggiator and clip sequencer.",
        features: &[
            "3 oscillators: wavetable, sample, granular, multisample (.sfz) and spectral, with 22 warp modes and unison up to 16 voices",
            "Wavetable editor: draw, harmonics, formulas, multi-frame editing, morphing, import / export of .wav tables",
            "Two filters (series or parallel) with 16 types plus a custom point filter drawn on the response graph",
            "4 envelopes and 8 LFOs, including real-time Lorenz / Rössler chaos; mod matrix with 33 sources and 16 slots",
            "Effect rack: up to 10 effects per bus, any order, two buses with several routings",
            "Arpeggiator with a 16-step pattern editor, clip sequencer with capture, MIDI out, key lock",
            "146 factory presets with categories, tags, favourites, ratings, HYBRID and RANDOM",
            "MIDI learn on every knob, Poly / Mono / Legato, undo / redo, drag and drop of tables and presets",
        ],
        shots: &[
            Shot { caption: "Overview", data: include_bytes!("../assets/synth_1.jpg") },
            Shot { caption: "Wavetable editor", data: include_bytes!("../assets/synth_2.jpg") },
            Shot { caption: "Mod matrix", data: include_bytes!("../assets/synth_3.jpg") },
            Shot { caption: "Effect rack", data: include_bytes!("../assets/synth_4.jpg") },
            Shot { caption: "Custom point filter", data: include_bytes!("../assets/synth_5.jpg") },
            Shot { caption: "Preset browser", data: include_bytes!("../assets/synth_6.jpg") },
        ],
    },
    PluginDef {
        id: "opto",
        name: "NyashkOpto",
        repo: "xslimakx/NyashkOpto",
        bundle: "NyashkOpto.vst3",
        category: Category::Effect,
        tagline: "Optical compressor",
        description: "LA-2A style optical compressor with program-dependent attack, two-stage release, \
                      compress / limit modes and analog hum + saturation.",
        tags: &["VST3", "Compressor", "Dynamics"],
        long: "A model of an optical (opto-cell) compressor in the spirit of the LA-2A. Attack is program-dependent and release is two-stage with memory, so it breathes like the hardware. Simple to use: Peak Reduction and Gain do most of the work.",
        features: &[
            "Peak Reduction: amount of compression; Gain: output level",
            "COMPRESS or LIMIT mode",
            "Hi Freq (HF Follow): emphasis in the sidechain to tame sibilance",
            "ANALOG: OFF / 50 Hz / 60 Hz soft saturation with mains hum, like real hardware",
            "Mix for parallel compression, Auto Makeup and Trim",
            "VU meter with INPUT, GR and OUTPUT views",
            "Transparent at Peak Reduction 0, click-free bypass",
        ],
        shots: &[
            Shot { caption: "Overview", data: include_bytes!("../assets/opto_1.jpg") },
        ],
    },
    PluginDef {
        id: "tune",
        name: "NyashkTune",
        repo: "xslimakx/NyashkTune",
        bundle: "NyashkTune.vst3",
        category: Category::Tool,
        tagline: "Realtime pitch correction",
        description: "Autotune from transparent natural correction to the hard T-Pain effect, with formant \
                      control, voices and a preset browser.",
        tags: &["VST3", "Autotune", "Vocal"],
        long: "Realtime vocal pitch correction, from transparent natural tuning to the hard T-Pain effect. YIN pitch detection with PSOLA resynthesis, 16 scales plus a custom one, formant control, harmony voices and a preset browser.",
        features: &[
            "YIN pitch detection with continuity tracking (no octave jumps), PSOLA resynthesis, mono and stereo",
            "16 scales plus Custom, key and reference A4; per-note BYPASS for blue notes and slides",
            "Retune Speed from natural to hard, Flex-Tune, Humanize, Natural Vibrato, synthetic vibrato with four shapes",
            "MIDI Target: tune to the notes you play",
            "Transpose, Detune and Formant Shift with 5 algorithms",
            "2 extra voices (harmony, octave, double) with interval, pan, detune and delay",
            "Air and Drive colour, voice ranges from Bass to Soprano",
            "28 presets (including Anime Vocal) and your own saved presets",
        ],
        shots: &[
            Shot { caption: "Tune", data: include_bytes!("../assets/tune_1.jpg") },
            Shot { caption: "Formant", data: include_bytes!("../assets/tune_2.jpg") },
            Shot { caption: "Voices", data: include_bytes!("../assets/tune_3.jpg") },
            Shot { caption: "Preset browser", data: include_bytes!("../assets/tune_4.jpg") },
        ],
    },
    PluginDef {
        id: "channel",
        name: "NyashkChannel",
        repo: "xslimakx/NyashkChannel",
        bundle: "NyashkChannel.vst3",
        category: Category::Effect,
        tagline: "Channel strip",
        description: "SSL-style channel strip: input stage, filters, expander / gate, compressor and a 4-band EQ in one plugin.",
        tags: &["VST3", "Channel strip", "EQ", "Dynamics"],
        long: "A complete console channel in one plugin, in the spirit of the SSL E / G channel: input section with line and mic gain, high-pass and low-pass filters, expander / gate, compressor with auto makeup, a four-band EQ (BROWN and BLACK voicing), and an output fader with width and mono.",
        features: &[
            "Input: polarity, -20 dB pad, line trim, mic gain, ANALOG saturation",
            "HP 18 dB/oct and LP filters, optionally on the dynamics sidechain only (SPLIT)",
            "Expander / gate with threshold, range, release and fast attack",
            "Compressor with soft knee, ratio up to infinity, release, fast attack and auto makeup",
            "Dynamics before or after the EQ",
            "4-band EQ: LF and HF shelf or bell, LMF and HMF with Q, BROWN / BLACK character",
            "Output fader, stereo width and mono, in / out and gain reduction meters",
        ],
        shots: &[
            Shot { caption: "Channel", data: include_bytes!("../assets/channel_1.jpg") },
        ],
    },
    PluginDef {
        id: "eq",
        name: "NyashkEQ",
        repo: "xslimakx/NyashkEQ",
        bundle: "NyashkEQ.vst3",
        category: Category::Effect,
        tagline: "Parametric equalizer",
        description: "4-band parametric EQ with HP / LP, draggable nodes over a live spectrum, mid / side mode and analog stage.",
        tags: &["VST3", "EQ"],
        long: "A clean four-band parametric equalizer in the style of a classic console EQ. Drag the nodes on the graph, see the live spectrum behind the curve, and work in stereo, mid or side. BLACK and BROWN voicings change how bands behave.",
        features: &[
            "Draggable nodes over a live spectrum analyser",
            "HP 18 dB/oct and LP filters",
            "LF and HF shelf or bell, two fully parametric mid bands",
            "STEREO, MID and SIDE processing",
            "BLACK and BROWN voicing, optional ANALOG stage",
            "Phase invert, output gain, click-free bypass",
        ],
        shots: &[
            Shot { caption: "Equalizer", data: include_bytes!("../assets/eq_1.jpg") },
        ],
    },
    PluginDef {
        id: "air",
        name: "NyashkAir",
        repo: "xslimakx/NyashkAir",
        bundle: "NyashkAir.vst3",
        category: Category::Effect,
        tagline: "Air / presence exciter",
        description: "Adds smooth air and presence to vocals and mixes without harshness.",
        tags: &["VST3", "Exciter", "Air"],
        long: "An air and presence exciter in the spirit of Slate Fresh Air. Two knobs add brightness and openness in the mids and highs, with dynamic reduction so it does not get harsh, and a live curve that shows exactly what is being added.",
        features: &[
            "MID AIR and HIGH AIR knobs, LINK to move them together",
            "Dynamic reduction keeps the top smooth",
            "Live curve of the added energy",
            "Input / output meters and trim, click-free power",
        ],
        shots: &[
            Shot { caption: "Air", data: include_bytes!("../assets/air_1.jpg") },
        ],
    },
    PluginDef {
        id: "pre",
        name: "NyashkPre",
        repo: "xslimakx/NyashkPre",
        bundle: "NyashkPre.vst3",
        category: Category::Effect,
        tagline: "Vocal preamp",
        description: "Light vocal preamp and saturator with tube, iron and clean character plus a simple tone section.",
        tags: &["VST3", "Preamp", "Saturation", "Vocal"],
        long: "A light vocal preamp: gain and drive into three characters (tube, iron transformer, clean solid-state) followed by low cut, warmth, presence and silk. Auto level keeps loudness steady while you set the sound.",
        features: &[
            "Gain and Drive with saturation curve display",
            "TUBE, IRON and CLEAN character",
            "Low cut, Warmth, Presence and Silk tone controls",
            "Parallel MIX, AUTO LEVEL, polarity, output",
        ],
        shots: &[
            Shot { caption: "Preamp", data: include_bytes!("../assets/pre_1.jpg") },
        ],
    },
    PluginDef {
        id: "meter",
        name: "NyashkMeter",
        repo: "xslimakx/NyashkMeter",
        bundle: "NyashkMeter.vst3",
        category: Category::Analysis,
        tagline: "Loudness and spectrum meter",
        description: "Master-bus analyser: LUFS (momentary, short-term, integrated), LRA, true peak, spectrum, stereo and tonal balance.",
        tags: &["VST3", "Meter", "LUFS", "Analyzer"],
        long: "A mastering meter that passes audio through untouched. ITU-R BS.1770-4 loudness with momentary, short-term and gated integrated LUFS, loudness range, true peak, a spectrum with selectable source, goniometer, correlation, loudness history and tonal balance.",
        features: &[
            "Momentary, short-term and integrated LUFS, LRA, max values",
            "Targets: -9, -14, -16, -23 LUFS with history line",
            "True peak (4x), sample peak, PLR and overs count",
            "Spectrum: curve or 1/3 octave, L+R / MID / SIDE / L / R, slopes, speed, hold",
            "Goniometer, correlation and balance",
            "Tonal balance across six bands",
            "Loudness history 1 or 5 minutes",
        ],
        shots: &[
            Shot { caption: "Overview", data: include_bytes!("../assets/meter_1.jpg") },
        ],
    },
];

// ---------------------------------------------------------------- platform

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Os {
    Linux,
    Windows,
    MacOs,
}

pub const OS: Os = if cfg!(target_os = "windows") {
    Os::Windows
} else if cfg!(target_os = "macos") {
    Os::MacOs
} else {
    Os::Linux
};

impl Os {
    pub fn name(self) -> &'static str {
        match self {
            Os::Linux => "Linux",
            Os::Windows => "Windows",
            Os::MacOs => "macOS",
        }
    }
    /// Does a release asset file name belong to this platform?
    pub fn matches_asset(self, name: &str) -> bool {
        let n = name.to_ascii_lowercase();
        if !n.ends_with(".zip") || n.contains("source") {
            return false;
        }
        match self {
            Os::Linux => n.contains("linux"),
            Os::Windows => n.contains("windows") || n.contains("-win"),
            Os::MacOs => n.contains("macos") || n.contains("mac"),
        }
    }
}

pub fn default_install_dir() -> PathBuf {
    match OS {
        Os::Windows => {
            let base = std::env::var_os("COMMONPROGRAMFILES")
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from(r"C:\Program Files\Common Files"));
            base.join("VST3")
        }
        Os::MacOs => dirs::home_dir().unwrap_or_default().join("Library/Audio/Plug-Ins/VST3"),
        Os::Linux => dirs::home_dir().unwrap_or_default().join(".vst3"),
    }
}

pub fn config_dir() -> PathBuf {
    let d = dirs::config_dir().unwrap_or_else(std::env::temp_dir).join("NyashkLauncher");
    let _ = std::fs::create_dir_all(&d);
    d
}

pub fn cache_dir() -> PathBuf {
    let d = dirs::cache_dir().unwrap_or_else(std::env::temp_dir).join("NyashkLauncher");
    let _ = std::fs::create_dir_all(&d);
    d
}

// ---------------------------------------------------------------- settings

#[derive(Serialize, Deserialize, Clone, PartialEq)]
#[serde(default)]
pub struct Settings {
    pub install_dir: String,
    pub check_on_startup: bool,
    pub include_prereleases: bool,
    pub keep_downloads: bool,
    pub github_token: String,
    pub background_minutes: u32,
    pub notify: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            install_dir: default_install_dir().to_string_lossy().into_owned(),
            check_on_startup: true,
            include_prereleases: false,
            keep_downloads: false,
            github_token: String::new(),
            background_minutes: 60,
            notify: true,
        }
    }
}

impl Settings {
    pub fn load() -> Settings {
        std::fs::read(config_dir().join("settings.json"))
            .ok()
            .and_then(|b| serde_json::from_slice(&b).ok())
            .unwrap_or_default()
    }
    pub fn save(&self) {
        if let Ok(b) = serde_json::to_vec_pretty(self) {
            let _ = std::fs::write(config_dir().join("settings.json"), b);
        }
    }
    pub fn dir(&self) -> PathBuf {
        let s = self.install_dir.trim();
        if s.is_empty() {
            default_install_dir()
        } else {
            PathBuf::from(s)
        }
    }
}

// ---------------------------------------------------------------- installed records

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct Record {
    pub version: String,
    pub dir: String,
    pub installed_at: String,
}

#[derive(Serialize, Deserialize, Default)]
pub struct Installed(pub HashMap<String, Record>);

impl Installed {
    pub fn load() -> Installed {
        std::fs::read(config_dir().join("installed.json"))
            .ok()
            .and_then(|b| serde_json::from_slice(&b).ok())
            .unwrap_or_default()
    }
    pub fn save(&self) {
        if let Ok(b) = serde_json::to_vec_pretty(self) {
            let _ = std::fs::write(config_dir().join("installed.json"), b);
        }
    }
}

pub fn bundle_path(dir: &Path, def: &PluginDef) -> PathBuf {
    dir.join(def.bundle)
}

// ---------------------------------------------------------------- versions

/// "v1.0.4" -> [1,0,4]; anything non-numeric sorts as 0.
pub fn parse_version(tag: &str) -> Vec<u64> {
    tag.trim()
        .trim_start_matches(['v', 'V'])
        .split(['.', '-', '+'])
        .map(|p| p.parse::<u64>().unwrap_or(0))
        .collect()
}

pub fn cmp_version(a: &str, b: &str) -> std::cmp::Ordering {
    let (mut x, mut y) = (parse_version(a), parse_version(b));
    let n = x.len().max(y.len());
    x.resize(n, 0);
    y.resize(n, 0);
    x.cmp(&y)
}

/// Current UTC time as `YYYY-MM-DD HH:MM` without extra dependencies.
pub fn now_stamp() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    let days = secs.div_euclid(86400);
    let rem = secs.rem_euclid(86400);
    // civil-from-days (Howard Hinnant)
    let z = days + 719468;
    let era = z.div_euclid(146097);
    let doe = z.rem_euclid(146097);
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    format!("{y:04}-{m:02}-{d:02} {:02}:{:02}", rem / 3600, (rem % 3600) / 60)
}

pub fn human_size(b: u64) -> String {
    if b >= 1 << 20 {
        format!("{:.1} MB", b as f64 / (1u64 << 20) as f64)
    } else {
        format!("{:.0} KB", b as f64 / 1024.0)
    }
}

/// Desktop notification through the platform tools (no extra dependencies).
pub fn notify(title: &str, body: &str) {
    use std::process::Command;
    let (t, b) = (title.replace('"', "'"), body.replace('"', "'"));
    let _ = match OS {
        Os::Linux => Command::new("notify-send").args([&t, &b]).spawn(),
        Os::MacOs => Command::new("osascript").args(["-e", &format!("display notification \"{b}\" with title \"{t}\"")]).spawn(),
        Os::Windows => Command::new("powershell")
            .args(["-NoProfile", "-WindowStyle", "Hidden", "-Command", &format!(
                "Add-Type -AssemblyName System.Windows.Forms; $n=New-Object System.Windows.Forms.NotifyIcon; \
                 $n.Icon=[System.Drawing.SystemIcons]::Information; $n.Visible=$true; \
                 $n.ShowBalloonTip(8000,'{t}','{b}',[System.Windows.Forms.ToolTipIcon]::Info); Start-Sleep 9; $n.Dispose()")])
            .spawn(),
    };
}

// ---------------------------------------------------------------- remote registry

pub const REGISTRY_BASE: &str = "https://raw.githubusercontent.com/xslimakx/NyashkLauncher/main/";

#[derive(Deserialize)]
struct RemoteShot {
    caption: String,
    url: String,
}
#[derive(Deserialize)]
struct RemotePlugin {
    id: String,
    name: String,
    repo: String,
    bundle: String,
    #[serde(default)]
    category: String,
    #[serde(default)]
    tagline: String,
    #[serde(default)]
    description: String,
    #[serde(default)]
    tags: Vec<String>,
    #[serde(default)]
    long: String,
    #[serde(default)]
    features: Vec<String>,
    #[serde(default)]
    shots: Vec<RemoteShot>,
}
#[derive(Deserialize)]
struct RemoteFile {
    plugins: Vec<RemotePlugin>,
}

fn leak(s: String) -> &'static str {
    Box::leak(s.into_boxed_str())
}

fn parse_category(s: &str) -> Category {
    match s.to_ascii_lowercase().as_str() {
        "instrument" | "instruments" => Category::Instrument,
        "tool" | "tools" | "vocal" | "vocal tools" => Category::Tool,
        "analysis" | "analyzer" | "meter" => Category::Analysis,
        _ => Category::Effect,
    }
}

pub fn shot_cache_path(url: &str) -> PathBuf {
    shot_cache(url)
}
fn shot_cache(url: &str) -> PathBuf {
    let safe: String = url.chars().map(|c| if c.is_ascii_alphanumeric() || c == '.' { c } else { '_' }).collect();
    cache_dir().join("shots").join(safe)
}
fn full_url(u: &str) -> String {
    if u.starts_with("http") {
        u.to_string()
    } else {
        format!("{REGISTRY_BASE}{}", u.trim_start_matches('/'))
    }
}

fn registry_cache() -> PathBuf {
    cache_dir().join("plugins.json")
}

/// The plugin list for this run: built-in entries, overridden / extended by the cached registry.
pub fn plugins() -> &'static [PluginDef] {
    static REG: std::sync::OnceLock<Vec<PluginDef>> = std::sync::OnceLock::new();
    REG.get_or_init(|| {
        let mut list: Vec<PluginDef> = BUILTIN.to_vec();
        let Some(rf) = std::fs::read(registry_cache()).ok().and_then(|b| serde_json::from_slice::<RemoteFile>(&b).ok()) else {
            return list;
        };
        for r in rf.plugins {
            let shots: Vec<Shot> = r
                .shots
                .iter()
                .filter_map(|s| {
                    let bytes = std::fs::read(shot_cache(&full_url(&s.url))).ok()?;
                    Some(Shot { caption: leak(s.caption.clone()), data: Box::leak(bytes.into_boxed_slice()) })
                })
                .collect();
            let old = list.iter().position(|d| d.id == r.id);
            let tags: Vec<&'static str> = r.tags.into_iter().map(leak).collect();
            let features: Vec<&'static str> = r.features.into_iter().map(leak).collect();
            let shots = if shots.is_empty() { old.map(|i| list[i].shots.to_vec()).unwrap_or_default() } else { shots };
            let def = PluginDef {
                id: leak(r.id),
                name: leak(r.name),
                repo: leak(r.repo),
                bundle: leak(r.bundle),
                category: parse_category(&r.category),
                tagline: leak(r.tagline),
                description: leak(r.description),
                tags: Box::leak(tags.into_boxed_slice()),
                long: leak(r.long),
                features: Box::leak(features.into_boxed_slice()),
                shots: Box::leak(shots.into_boxed_slice()),
            };
            match old {
                Some(i) => list[i] = def,
                None => list.push(def),
            }
        }
        list
    })
}
