#![cfg_attr(all(not(debug_assertions), target_os = "windows"), windows_subsystem = "windows")]

mod github;
mod model;
mod ui;

use eframe::egui;
use github::Release;
use model::*;
use std::sync::mpsc::{channel, Receiver, Sender};

pub enum Msg {
    Releases(usize, Result<Vec<Release>, String>),
    Progress(usize, f32, String),
    Installed(usize, Result<String, String>),
    Launcher(Result<Vec<Release>, String>),
    SelfUpdated(Result<String, String>),
}

#[derive(Default)]
pub struct PluginState {
    pub releases: Vec<Release>, // raw, newest first
    pub checking: bool,
    pub checked: bool,
    pub error: Option<String>,
    pub chosen: Option<String>, // version picked in the dropdown (None = latest)
    pub tick: bool,             // included in "Update selected"
    pub busy: Option<(f32, String)>,
    pub on_disk: bool,
}

#[derive(PartialEq, Clone, Copy)]
pub enum Page {
    All,
    Cat(Category),
    Detail(usize),
    Settings,
}

pub struct App {
    pub settings: Settings,
    pub installed: Installed,
    pub states: Vec<PluginState>,
    pub page: Page,
    pub status: String,
    pub last_checked: String,
    pub tx: Sender<Msg>,
    rx: Receiver<Msg>,
    pub ctx: egui::Context,
    was_focused: bool,
    pub tex: std::collections::HashMap<String, egui::TextureHandle>,
    pub shot: usize,
    pub launcher: Option<Release>,      // newer launcher release, if any
    pub launcher_busy: Option<(f32, String)>,
    pub launcher_restart: bool,
    last_auto: std::time::Instant,
    background: bool,
    notified: std::collections::HashSet<String>,
}

#[derive(PartialEq, Clone, Copy)]
pub enum Status {
    Checking,
    NotInstalled,
    UpToDate,
    Update,
    UnknownVersion,
    NoBuild,
    Offline,
}

impl App {
    fn new(cc: &eframe::CreationContext) -> App {
        ui::apply_theme(&cc.egui_ctx);
        let (tx, rx) = channel();
        let settings = Settings::load();
        let mut app = App {
            installed: Installed::load(),
            states: PLUGINS.iter().map(|_| PluginState::default()).collect(),
            page: match std::env::var("NYASHK_PAGE").as_deref() {
                Ok("settings") => Page::Settings,
                Ok("synth") => Page::Detail(0),
                Ok("tune") => Page::Detail(2),
                _ => Page::All,
            },
            status: "Ready".into(),
            last_checked: "never".into(),
            settings,
            tx,
            rx,
            ctx: cc.egui_ctx.clone(),
            was_focused: true,
            tex: Default::default(),
            shot: 0,
            launcher: None,
            launcher_busy: None,
            launcher_restart: false,
            last_auto: std::time::Instant::now(),
            background: false,
            notified: Default::default(),
        };
        app.refresh_disk();
        if app.settings.check_on_startup {
            app.check_all();
        }
        app
    }

    pub fn refresh_disk(&mut self) {
        let dir = self.settings.dir();
        for (i, def) in PLUGINS.iter().enumerate() {
            self.states[i].on_disk = bundle_path(&dir, def).exists();
        }
    }

    /// Version we know is installed in the *current* folder.
    pub fn installed_version(&self, i: usize) -> Option<String> {
        if !self.states[i].on_disk {
            return None;
        }
        let r = self.installed.0.get(PLUGINS[i].id)?;
        (r.dir == self.settings.dir().to_string_lossy()).then(|| r.version.clone())
    }

    /// Releases the user may pick: not drafts, pre-releases per setting, with a build for this OS.
    pub fn versions(&self, i: usize) -> Vec<&Release> {
        let mut v: Vec<&Release> = self.states[i]
            .releases
            .iter()
            .filter(|r| !r.draft && (self.settings.include_prereleases || !r.prerelease) && r.asset().is_some())
            .collect();
        v.sort_by(|a, b| cmp_version(&b.tag_name, &a.tag_name));
        v
    }

    pub fn latest(&self, i: usize) -> Option<&Release> {
        self.versions(i).into_iter().next()
    }

    pub fn status(&self, i: usize) -> Status {
        let st = &self.states[i];
        if st.checking && st.releases.is_empty() {
            return Status::Checking;
        }
        let Some(latest) = self.latest(i) else {
            return if st.error.is_some() {
                Status::Offline
            } else if st.checked {
                Status::NoBuild
            } else {
                Status::Checking
            };
        };
        if !st.on_disk {
            return Status::NotInstalled;
        }
        match self.installed_version(i) {
            None => Status::UnknownVersion,
            Some(v) if cmp_version(&latest.tag_name, &v).is_gt() => Status::Update,
            Some(_) => Status::UpToDate,
        }
    }

    pub fn check_all(&mut self) {
        let token = self.settings.github_token.clone();
        for (i, def) in PLUGINS.iter().enumerate() {
            if self.states[i].checking {
                continue;
            }
            self.states[i].checking = true;
            self.states[i].error = None;
            let (tx, ctx, token) = (self.tx.clone(), self.ctx.clone(), token.clone());
            std::thread::spawn(move || {
                let _ = tx.send(Msg::Releases(i, github::fetch_releases(def.repo, &token)));
                ctx.request_repaint();
            });
        }
        let (tx, ctx) = (self.tx.clone(), self.ctx.clone());
        let token = self.settings.github_token.clone();
        std::thread::spawn(move || {
            let _ = tx.send(Msg::Launcher(github::fetch_releases(LAUNCHER_REPO, &token)));
            ctx.request_repaint();
        });
        self.last_auto = std::time::Instant::now();
        self.status = "Checking for updates…".into();
    }

    pub fn update_launcher(&mut self) {
        let Some(rel) = self.launcher.clone() else { return };
        if self.launcher_busy.is_some() {
            return;
        }
        let (tx, ctx) = (self.tx.clone(), self.ctx.clone());
        let token = self.settings.github_token.clone();
        self.launcher_busy = Some((0.0, "Starting…".into()));
        std::thread::spawn(move || {
            let send = |m: Msg| {
                let _ = tx.send(m);
                ctx.request_repaint();
            };
            let res = (|| {
                let asset = rel.asset().ok_or_else(|| format!("no launcher build for {}", OS.name()))?;
                let zip = github::download(asset, &token, |d, t| {
                    send(Msg::Progress(usize::MAX, d as f32 / t.max(1) as f32, format!("Downloading {}", human_size(d))))
                })?;
                github::self_update(&zip)?;
                let _ = std::fs::remove_file(&zip);
                Ok::<String, String>(rel.tag_name.clone())
            })();
            send(Msg::SelfUpdated(res));
        });
    }

    pub fn restart(&self) {
        if let Ok(exe) = std::env::current_exe() {
            let _ = std::process::Command::new(exe).spawn();
        }
        std::process::exit(0);
    }

    pub fn install(&mut self, i: usize, rel: Release) {
        if self.states[i].busy.is_some() {
            return;
        }
        let def = &PLUGINS[i];
        let (dir, token, keep) = (self.settings.dir(), self.settings.github_token.clone(), self.settings.keep_downloads);
        let (tx, ctx) = (self.tx.clone(), self.ctx.clone());
        self.states[i].busy = Some((0.0, "Starting…".into()));
        std::thread::spawn(move || {
            let send = |m: Msg| {
                let _ = tx.send(m);
                ctx.request_repaint();
            };
            let res = (|| {
                let asset = rel.asset().ok_or_else(|| format!("{} has no build for {}", rel.tag_name, OS.name()))?;
                let zip = github::download(asset, &token, |d, t| {
                    send(Msg::Progress(
                        i,
                        d as f32 / t.max(1) as f32,
                        format!("Downloading {} / {}", human_size(d), human_size(t.max(d))),
                    ))
                })?;
                send(Msg::Progress(i, 1.0, "Installing…".into()));
                github::install_zip(&zip, def, &dir)?;
                if !keep {
                    let _ = std::fs::remove_file(&zip);
                }
                Ok::<String, String>(rel.tag_name.clone())
            })();
            send(Msg::Installed(i, res));
        });
    }

    pub fn uninstall(&mut self, i: usize) {
        let def = &PLUGINS[i];
        match github::uninstall(def, &self.settings.dir()) {
            Ok(()) => {
                self.installed.0.remove(def.id);
                self.installed.save();
                self.status = format!("{} removed", def.name);
            }
            Err(e) => self.status = format!("Could not remove {}: {e}", def.name),
        }
        self.refresh_disk();
    }

    pub fn updates_available(&self) -> Vec<usize> {
        (0..PLUGINS.len()).filter(|&i| self.status(i) == Status::Update && self.states[i].busy.is_none()).collect()
    }

    pub fn update_many(&mut self, which: Vec<usize>) {
        for i in which {
            if let Some(rel) = self.latest(i).cloned() {
                self.states[i].chosen = None;
                self.states[i].tick = false;
                self.install(i, rel);
            }
        }
    }

    fn pump(&mut self) {
        while let Ok(m) = self.rx.try_recv() {
            match m {
                Msg::Releases(i, res) => {
                    let st = &mut self.states[i];
                    st.checking = false;
                    st.checked = true;
                    match res {
                        Ok(r) => st.releases = r,
                        Err(e) => st.error = Some(e),
                    }
                    if self.states.iter().all(|s| !s.checking) {
                        self.last_checked = now_stamp() + " UTC";
                        let ups = self.updates_available();
                        let n = ups.len();
                        if self.background && self.settings.notify {
                            for &u in &ups {
                                let tag = self.latest(u).map(|r| r.tag_name.clone()).unwrap_or_default();
                                if self.notified.insert(format!("{}:{tag}", PLUGINS[u].id)) {
                                    notify("Nyashk plugin update", &format!("{} {tag} is available", PLUGINS[u].name));
                                }
                            }
                        }
                        self.background = false;
                        let err = self.states.iter().find_map(|s| s.error.clone());
                        self.status = match (err, n) {
                            (Some(e), _) => e,
                            (None, 0) => "Everything is up to date".into(),
                            (None, n) => format!("{n} update{} available", if n == 1 { "" } else { "s" }),
                        };
                    }
                }
                Msg::Progress(usize::MAX, f, text) => self.launcher_busy = Some((f, text)),
                Msg::Progress(i, f, text) => self.states[i].busy = Some((f, text)),
                Msg::Launcher(res) => {
                    if let Ok(list) = res {
                        let mut v: Vec<&Release> = list
                            .iter()
                            .filter(|r| !r.draft && !r.prerelease && r.asset().is_some())
                            .collect();
                        v.sort_by(|a, b| cmp_version(&b.tag_name, &a.tag_name));
                        self.launcher = v
                            .first()
                            .filter(|r| cmp_version(&r.tag_name, env!("CARGO_PKG_VERSION")).is_gt())
                            .map(|r| (*r).clone());
                        if let (Some(r), true) = (&self.launcher, self.background && self.settings.notify) {
                            if self.notified.insert(format!("launcher:{}", r.tag_name)) {
                                notify("Nyashk Launcher", &format!("Launcher {} is available", r.tag_name));
                            }
                        }
                    }
                }
                Msg::SelfUpdated(res) => {
                    self.launcher_busy = None;
                    match res {
                        Ok(tag) => {
                            self.launcher_restart = true;
                            self.status = format!("Launcher updated to {tag}. Restart to use it.");
                        }
                        Err(e) => self.status = format!("Launcher update failed: {e}"),
                    }
                }
                Msg::Installed(i, res) => {
                    self.states[i].busy = None;
                    let def = &PLUGINS[i];
                    match res {
                        Ok(tag) => {
                            self.installed.0.insert(
                                def.id.into(),
                                Record {
                                    version: tag.clone(),
                                    dir: self.settings.dir().to_string_lossy().into_owned(),
                                    installed_at: now_stamp(),
                                },
                            );
                            self.installed.save();
                            self.status =
                                format!("{} {} installed. Rescan plugins in your DAW if it is open.", def.name, tag);
                        }
                        Err(e) => self.status = format!("{}: {e}", def.name),
                    }
                    self.refresh_disk();
                }
            }
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.pump();
        // re-read the plugin folder when the window regains focus (DAW / file manager changes)
        let focused = ctx.input(|i| i.viewport().focused.unwrap_or(true));
        if focused && !self.was_focused {
            self.refresh_disk();
        }
        self.was_focused = focused;
        // background re-check while the launcher is open
        let mins = self.settings.background_minutes as u64;
        if mins > 0 {
            let every = std::time::Duration::from_secs(mins * 60);
            if self.last_auto.elapsed() >= every && !self.states.iter().any(|s| s.checking) {
                self.background = true;
                self.check_all();
            }
            ctx.request_repaint_after(std::time::Duration::from_secs(30));
        }
        // window title shows pending updates
        let n = self.updates_available().len();
        let title = if n > 0 { format!("Nyashk Launcher ({n} update{})", if n == 1 { "" } else { "s" }) } else { "Nyashk Launcher".into() };
        ctx.send_viewport_cmd(egui::ViewportCommand::Title(title));
        ui::draw(self, ctx);
    }
}

fn main() -> eframe::Result {
    #[cfg(target_os = "linux")]
    linux_integration();
    if let Ok(exe) = std::env::current_exe() {
        let _ = std::fs::remove_file(exe.with_extension("old")); // leftover of a Windows self-update
    }
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Nyashk Launcher")
            .with_app_id("NyashkLauncher")
            .with_inner_size(start_size())
            .with_min_inner_size([980.0, 560.0])
            .with_icon(window_icon()),
        ..Default::default()
    };
    eframe::run_native("Nyashk Launcher", options, Box::new(|cc| Ok(Box::new(App::new(cc)))))
}

pub const ICON_PNG: &[u8] = include_bytes!("../assets/icon.png");

fn window_icon() -> egui::IconData {
    let img = image::load_from_memory(ICON_PNG).expect("icon").to_rgba8();
    egui::IconData { width: img.width(), height: img.height(), rgba: img.into_raw() }
}

/// Default window size; `NYASHK_SIZE=WxH` is a dev hook used for documentation screenshots.
fn start_size() -> [f32; 2] {
    std::env::var("NYASHK_SIZE")
        .ok()
        .and_then(|v| {
            let (w, h) = v.split_once('x')?;
            Some([w.parse().ok()?, h.parse().ok()?])
        })
        .unwrap_or([1120.0, 720.0])
}

/// Linux taskbars take the icon from a .desktop file, not from the window.
/// Create / refresh ~/.local/share/applications/NyashkLauncher.desktop and its icon.
#[cfg(target_os = "linux")]
fn linux_integration() {
    let (Some(data), Ok(exe)) = (dirs::data_dir(), std::env::current_exe()) else { return };
    let icon = data.join("icons/hicolor/256x256/apps/NyashkLauncher.png");
    if std::fs::read(&icon).map(|b| b != ICON_PNG).unwrap_or(true) {
        if let Some(d) = icon.parent() {
            let _ = std::fs::create_dir_all(d);
        }
        let _ = std::fs::write(&icon, ICON_PNG);
        let _ = std::process::Command::new("gtk-update-icon-cache")
            .args(["-q", "-t", "-f"])
            .arg(data.join("icons/hicolor"))
            .status();
    }
    let desktop = format!(
        "[Desktop Entry]\nType=Application\nName=Nyashk Launcher\nComment=Install and update Nyashk plugins\n\
         Exec=\"{}\"\nIcon=NyashkLauncher\nTerminal=false\nCategories=AudioVideo;Audio;\nStartupWMClass=NyashkLauncher\n",
        exe.display()
    );
    let path = data.join("applications/NyashkLauncher.desktop");
    if std::fs::read_to_string(&path).map(|c| c != desktop).unwrap_or(true) {
        if let Some(d) = path.parent() {
            let _ = std::fs::create_dir_all(d);
        }
        let _ = std::fs::write(&path, desktop);
        let _ = std::process::Command::new("update-desktop-database").arg(data.join("applications")).status();
    }
}
