//! Monochrome (black / white / silver) interface, same language as the Nyashk plugins.

use crate::github::Release;
use crate::model::*;
use crate::{App, Page, Status, ICON_PNG};
use eframe::egui::{self, Align, Color32, CornerRadius, Context, Frame, Layout, Margin, RichText, Stroke, Ui};

const BG: Color32 = Color32::from_rgb(12, 12, 12);
const PANEL: Color32 = Color32::from_rgb(19, 19, 19);
const CARD: Color32 = Color32::from_rgb(22, 22, 22);
const LINE: Color32 = Color32::from_rgb(44, 44, 44);
const TEXT: Color32 = Color32::from_rgb(222, 222, 222);
const DIM: Color32 = Color32::from_rgb(138, 138, 138);
const LIGHT: Color32 = Color32::from_rgb(228, 228, 228);
const WHITE: Color32 = Color32::from_rgb(250, 250, 250);
const BLACK: Color32 = Color32::from_rgb(10, 10, 10);

enum Action {
    Check,
    Install(usize, Release),
    Uninstall(usize),
    UpdateMany(Vec<usize>),
    UpdateSelf,
    Open(usize),
}

pub fn apply_theme(ctx: &Context) {
    let mut v = egui::Visuals::dark();
    v.panel_fill = BG;
    v.window_fill = PANEL;
    v.extreme_bg_color = Color32::from_rgb(9, 9, 9);
    v.faint_bg_color = Color32::from_rgb(20, 20, 20);
    v.selection.bg_fill = Color32::from_rgb(200, 200, 200);
    v.selection.stroke = Stroke::new(1.0, BLACK);
    v.widgets.noninteractive.bg_stroke = Stroke::new(1.0, LINE);
    v.widgets.noninteractive.fg_stroke = Stroke::new(1.0, TEXT);
    for (w, fill) in [
        (&mut v.widgets.inactive, Color32::from_rgb(30, 30, 30)),
        (&mut v.widgets.hovered, Color32::from_rgb(46, 46, 46)),
        (&mut v.widgets.active, Color32::from_rgb(60, 60, 60)),
        (&mut v.widgets.open, Color32::from_rgb(38, 38, 38)),
    ] {
        w.bg_fill = fill;
        w.weak_bg_fill = fill;
        w.bg_stroke = Stroke::new(1.0, Color32::from_rgb(62, 62, 62));
        w.fg_stroke = Stroke::new(1.0, TEXT);
        w.corner_radius = CornerRadius::same(6);
        w.expansion = 0.0;
    }
    v.widgets.hovered.bg_stroke = Stroke::new(1.0, Color32::from_rgb(110, 110, 110));
    ctx.set_visuals(v);
    ctx.style_mut(|s| {
        s.spacing.item_spacing = egui::vec2(8.0, 8.0);
        s.spacing.button_padding = egui::vec2(14.0, 6.0);
        s.spacing.interact_size.y = 30.0;
        s.text_styles.insert(egui::TextStyle::Body, egui::FontId::proportional(14.0));
        s.text_styles.insert(egui::TextStyle::Button, egui::FontId::proportional(14.0));
        s.text_styles.insert(egui::TextStyle::Small, egui::FontId::proportional(12.0));
    });
}

// ------------------------------------------------------------------ widgets

fn primary(ui: &mut Ui, text: &str) -> egui::Response {
    ui.scope(|ui| {
        let w = &mut ui.visuals_mut().widgets;
        for (s, fill) in [(&mut w.inactive, LIGHT), (&mut w.hovered, WHITE), (&mut w.active, Color32::from_rgb(190, 190, 190))] {
            s.weak_bg_fill = fill;
            s.bg_fill = fill;
            s.bg_stroke = Stroke::NONE;
            s.fg_stroke = Stroke::new(1.0, BLACK);
        }
        ui.add(egui::Button::new(RichText::new(text).strong()))
    })
    .inner
}

fn secondary(ui: &mut Ui, text: &str) -> egui::Response {
    ui.add(egui::Button::new(text))
}

fn badge(ui: &mut Ui, text: &str, filled: bool) {
    let (fill, fg, stroke) = if filled {
        (LIGHT, BLACK, Stroke::NONE)
    } else {
        (Color32::TRANSPARENT, DIM, Stroke::new(1.0, Color32::from_rgb(70, 70, 70)))
    };
    Frame::new().fill(fill).stroke(stroke).corner_radius(10).inner_margin(Margin::symmetric(9, 2)).show(ui, |ui| {
        ui.label(RichText::new(text).size(11.5).strong().color(fg));
    });
}

fn nav_item(ui: &mut Ui, text: &str, count: usize, selected: bool) -> bool {
    let (rect, resp) = ui.allocate_exact_size(egui::vec2(ui.available_width(), 34.0), egui::Sense::click());
    let hovered = resp.hovered();
    if selected {
        ui.painter().rect_filled(rect, 6.0, LIGHT);
    } else if hovered {
        ui.painter().rect_filled(rect, 6.0, Color32::from_rgb(30, 30, 30));
    }
    let fg = if selected { BLACK } else { TEXT };
    ui.painter().text(rect.left_center() + egui::vec2(12.0, 0.0), egui::Align2::LEFT_CENTER, text, egui::FontId::proportional(14.0), fg);
    if count > 0 {
        let c = rect.right_center() - egui::vec2(18.0, 0.0);
        ui.painter().circle_filled(c, 9.0, if selected { BLACK } else { LIGHT });
        ui.painter().text(c, egui::Align2::CENTER_CENTER, count.to_string(), egui::FontId::proportional(11.0), if selected { LIGHT } else { BLACK });
    }
    resp.clicked()
}

/// Very small Markdown subset: headings, bullets, **bold**, `code`, rules.
fn markdown(ui: &mut Ui, text: &str) {
    ui.spacing_mut().item_spacing.y = 3.0;
    for line in text.lines() {
        let l = line.trim_end();
        let t = l.trim_start();
        if t.is_empty() {
            ui.add_space(3.0);
        } else if t.starts_with("---") {
            ui.separator();
        } else if let Some(h) = t.strip_prefix("### ").or_else(|| t.strip_prefix("## ")).or_else(|| t.strip_prefix("# ")) {
            ui.add_space(2.0);
            ui.label(RichText::new(h.replace("**", "")).strong().size(14.0).color(WHITE));
        } else {
            let (bullet, body) = match t.strip_prefix("- ").or_else(|| t.strip_prefix("* ")) {
                Some(b) => (true, b),
                None => (false, t),
            };
            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing.x = 0.0;
                if bullet {
                    ui.label(RichText::new("•  ").color(DIM));
                }
                let body = body.replace('`', "");
                for (k, seg) in body.split("**").enumerate() {
                    if seg.is_empty() {
                        continue;
                    }
                    let rt = RichText::new(seg);
                    ui.label(if k % 2 == 1 { rt.strong().color(WHITE) } else { rt.color(Color32::from_rgb(190, 190, 190)) });
                }
            });
        }
    }
}

fn open_path(p: &std::path::Path) {
    let cmd = match OS {
        Os::Linux => "xdg-open",
        Os::MacOs => "open",
        Os::Windows => "explorer",
    };
    let _ = std::process::Command::new(cmd).arg(p).spawn();
}

fn dir_size(p: &std::path::Path) -> u64 {
    std::fs::read_dir(p)
        .map(|rd| {
            rd.flatten()
                .map(|e| match e.metadata() {
                    Ok(m) if m.is_dir() => dir_size(&e.path()),
                    Ok(m) => m.len(),
                    _ => 0,
                })
                .sum()
        })
        .unwrap_or(0)
}

// ------------------------------------------------------------------ main layout

pub fn draw(app: &mut App, ctx: &Context) {
    let mut acts: Vec<Action> = Vec::new();
    let busy_any = app.states.iter().any(|s| s.busy.is_some());
    let checking_any = app.states.iter().any(|s| s.checking);

    egui::TopBottomPanel::top("top")
        .exact_height(62.0)
        .frame(Frame::new().fill(PANEL).inner_margin(Margin::symmetric(22, 0)).stroke(Stroke::new(1.0, LINE)))
        .show(ctx, |ui| {
            ui.horizontal_centered(|ui| {
                if let Some((id, _)) = get_tex(app, ui.ctx(), "icon", ICON_PNG) {
                    ui.add(egui::Image::new((id, egui::vec2(34.0, 34.0))).corner_radius(6));
                }
                ui.label(RichText::new("NYASHK").size(22.0).strong().color(WHITE));
                ui.label(RichText::new("LAUNCHER").size(12.0).color(DIM));
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    let all = app.updates_available();
                    let ticked: Vec<usize> = all.iter().copied().filter(|&i| app.states[i].tick).collect();
                    let n = all.len();
                    ui.add_enabled_ui(n > 0 && !busy_any, |ui| {
                        if primary(ui, &format!("Update all ({n})")).clicked() {
                            acts.push(Action::UpdateMany(all.clone()));
                        }
                    });
                    ui.add_enabled_ui(!ticked.is_empty() && !busy_any, |ui| {
                        if secondary(ui, &format!("Update selected ({})", ticked.len())).clicked() {
                            acts.push(Action::UpdateMany(ticked.clone()));
                        }
                    });
                    ui.add_enabled_ui(!checking_any, |ui| {
                        let label = if checking_any { "Checking…" } else { "Check for updates" };
                        if secondary(ui, label).clicked() {
                            acts.push(Action::Check);
                        }
                    });
                });
            });
        });

    if let Some(rel) = app.launcher.clone() {
        egui::TopBottomPanel::top("banner")
            .exact_height(40.0)
            .frame(Frame::new().fill(Color32::from_rgb(34, 34, 34)).inner_margin(Margin::symmetric(22, 0)).stroke(Stroke::new(1.0, LINE)))
            .show(ctx, |ui| {
                ui.horizontal_centered(|ui| {
                    ui.label(RichText::new(format!("Launcher {} is available (you have v{})", rel.tag_name, env!("CARGO_PKG_VERSION"))).color(WHITE));
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if app.launcher_restart {
                            if primary(ui, "Restart now").clicked() {
                                app.restart();
                            }
                        } else if let Some((f, t)) = &app.launcher_busy {
                            ui.add(egui::ProgressBar::new(*f).desired_width(220.0).fill(LIGHT).text(RichText::new(t).size(12.0).color(Color32::from_rgb(40, 40, 40))));
                        } else if primary(ui, "Update launcher").clicked() {
                            acts.push(Action::UpdateSelf);
                        }
                    });
                });
            });
    }

    egui::TopBottomPanel::bottom("status")
        .exact_height(30.0)
        .frame(Frame::new().fill(PANEL).inner_margin(Margin::symmetric(22, 0)).stroke(Stroke::new(1.0, LINE)))
        .show(ctx, |ui| {
            ui.horizontal_centered(|ui| {
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.label(RichText::new(format!("Last check: {}", app.last_checked)).size(12.0).color(DIM));
                    ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                        ui.add(egui::Label::new(RichText::new(&app.status).size(12.5).color(TEXT)).truncate());
                    });
                });
            });
        });

    egui::SidePanel::left("nav")
        .exact_width(208.0)
        .resizable(false)
        .frame(Frame::new().fill(PANEL).inner_margin(Margin::same(12)).stroke(Stroke::new(1.0, LINE)))
        .show(ctx, |ui| {
            ui.spacing_mut().item_spacing.y = 4.0;
            ui.add_space(6.0);
            ui.label(RichText::new("LIBRARY").size(11.0).color(DIM));
            let upd_all = app.updates_available().len();
            if nav_item(ui, "All plugins", upd_all, app.page == Page::All) {
                app.page = Page::All;
            }
            for c in Category::ALL {
                let n = plugins().iter().enumerate().filter(|(i, d)| d.category == c && app.status(*i) == Status::Update).count();
                if nav_item(ui, c.label(), n, app.page == Page::Cat(c)) {
                    app.page = Page::Cat(c);
                }
            }
            ui.add_space(14.0);
            ui.label(RichText::new("LAUNCHER").size(11.0).color(DIM));
            if nav_item(ui, "Settings", 0, app.page == Page::Settings) {
                app.page = Page::Settings;
            }
            ui.with_layout(Layout::bottom_up(Align::Min), |ui| {
                ui.label(RichText::new(format!("v{}  ·  {}", env!("CARGO_PKG_VERSION"), OS.name())).size(11.5).color(DIM));
                ui.label(RichText::new(short_path(&app.settings.dir().to_string_lossy())).size(11.5).color(Color32::from_rgb(170, 170, 170)));
                ui.label(RichText::new("Installing to").size(11.0).color(DIM));
            });
        });

    egui::CentralPanel::default().frame(Frame::new().fill(BG).inner_margin(Margin::symmetric(24, 18))).show(ctx, |ui| {
        egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
            if app.page == Page::Settings {
                settings_page(app, ui);
            } else if let Page::Detail(i) = app.page {
                detail_page(app, ui, i, &mut acts);
            } else {
                let cats: Vec<Category> = match app.page {
                    Page::Cat(c) => vec![c],
                    _ => Category::ALL.to_vec(),
                };
                for c in cats {
                    ui.label(RichText::new(c.label().to_uppercase()).size(12.0).color(DIM));
                    ui.add_space(2.0);
                    for i in 0..plugins().len() {
                        if plugins()[i].category == c {
                            card(app, ui, i, &mut acts);
                            ui.add_space(6.0);
                        }
                    }
                    ui.add_space(10.0);
                }
            }
        });
    });

    for a in acts {
        match a {
            Action::Check => app.check_all(),
            Action::Install(i, r) => app.install(i, r),
            Action::Uninstall(i) => app.uninstall(i),
            Action::UpdateMany(v) => app.update_many(v),
            Action::UpdateSelf => app.update_launcher(),
            Action::Open(i) => {
                app.page = Page::Detail(i);
                app.shot = 0;
            }
        }
    }
}

fn short_path(p: &str) -> String {
    if p.chars().count() <= 30 {
        return p.into();
    }
    let tail: String = p.chars().rev().take(28).collect::<Vec<_>>().into_iter().rev().collect();
    format!("…{tail}")
}

// ------------------------------------------------------------------ images

fn get_tex(app: &mut App, ctx: &Context, key: &str, bytes: &[u8]) -> Option<(egui::TextureId, egui::Vec2)> {
    if !app.tex.contains_key(key) {
        let img = image::load_from_memory(bytes).ok()?.to_rgba8();
        let ci = egui::ColorImage::from_rgba_unmultiplied([img.width() as usize, img.height() as usize], img.as_raw());
        app.tex.insert(key.to_string(), ctx.load_texture(key, ci, egui::TextureOptions::LINEAR));
    }
    let t = app.tex.get(key)?;
    Some((t.id(), t.size_vec2()))
}

/// Draw a screenshot scaled to fit `max`, keeping its aspect ratio. Returns the click response.
fn picture(app: &mut App, ui: &mut Ui, key: &str, bytes: &[u8], max: egui::Vec2) -> Option<egui::Response> {
    let (id, size) = get_tex(app, ui.ctx(), key, bytes)?;
    let scale = (max.x / size.x).min(max.y / size.y);
    let shown = size * scale;
    let resp = ui.add(egui::Image::new((id, shown)).corner_radius(6).sense(egui::Sense::click()));
    Some(resp.on_hover_cursor(egui::CursorIcon::PointingHand))
}

// ------------------------------------------------------------------ plugin card

fn card(app: &mut App, ui: &mut Ui, i: usize, acts: &mut Vec<Action>) {
    let def = &plugins()[i];
    let status = app.status(i);
    Frame::new().fill(CARD).stroke(Stroke::new(1.0, LINE)).corner_radius(10).inner_margin(Margin::same(16)).show(ui, |ui| {
        ui.set_width(ui.available_width());
        let right_w = 262.0;
        let thumb_w = 196.0;
        let left_w = (ui.available_width() - right_w - thumb_w - 40.0).max(200.0);
        ui.horizontal_top(|ui| {
            // ---- preview picture: click opens the plugin page
            if let Some(s) = def.shots.first() {
                if let Some(r) = picture(app, ui, &format!("{}:0", def.id), s.data, egui::vec2(thumb_w, 124.0)) {
                    if r.clicked() {
                        acts.push(Action::Open(i));
                    }
                }
            }
            // ---- identity
            ui.allocate_ui_with_layout(egui::vec2(left_w, 0.0), Layout::top_down(Align::Min), |ui| {
                ui.horizontal(|ui| {
                    if status == Status::Update {
                        ui.checkbox(&mut app.states[i].tick, "").on_hover_text("Include in “Update selected”");
                    }
                    let r = ui.add(egui::Label::new(RichText::new(def.name).size(20.0).strong().color(WHITE)).sense(egui::Sense::click()));
                    if r.on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
                        acts.push(Action::Open(i));
                    }
                });
                ui.label(RichText::new(def.tagline).size(13.0).color(DIM));
                ui.label(RichText::new(def.description).size(13.0).color(Color32::from_rgb(175, 175, 175)));
                ui.add_space(2.0);
                ui.horizontal(|ui| {
                    for t in def.tags {
                        badge(ui, t, false);
                    }
                });
                if secondary(ui, "Details").clicked() {
                    acts.push(Action::Open(i));
                }
            });
            controls(app, ui, i, acts, right_w);
        });
        notes(app, ui, i);
    });
}

/// Status, versions, version picker and install / update buttons.
fn controls(app: &mut App, ui: &mut Ui, i: usize, acts: &mut Vec<Action>, right_w: f32) {
    let status = app.status(i);
    let inst = app.installed_version(i);
    let vers: Vec<Release> = app.versions(i).into_iter().cloned().collect();
    let latest_tag = vers.first().map(|r| r.tag_name.clone());
    let chosen = app.states[i].chosen.clone().filter(|t| vers.iter().any(|r| &r.tag_name == t)).or(latest_tag.clone());
    let busy = app.states[i].busy.clone();
    let on_disk = app.states[i].on_disk;
    let err = app.states[i].error.clone();

    ui.allocate_ui_with_layout(egui::vec2(right_w, 0.0), Layout::top_down(Align::Min), |ui| {
        ui.horizontal(|ui| {
            match status {
                Status::Checking => badge(ui, "CHECKING…", false),
                Status::NotInstalled => badge(ui, "NOT INSTALLED", false),
                Status::UpToDate => badge(ui, "UP TO DATE", false),
                Status::Update => badge(ui, "UPDATE AVAILABLE", true),
                Status::UnknownVersion => badge(ui, "INSTALLED · VERSION UNKNOWN", false),
                Status::NoBuild => badge(ui, "NO BUILD FOR THIS OS", false),
                Status::Offline => badge(ui, "OFFLINE", false),
            };
        });
        egui::Grid::new(("ver", i)).num_columns(2).spacing([10.0, 3.0]).show(ui, |ui| {
            ui.label(RichText::new("Installed").size(12.5).color(DIM));
            ui.label(RichText::new(inst.clone().unwrap_or_else(|| if on_disk { "unknown".into() } else { "—".into() })).color(TEXT));
            ui.end_row();
            ui.label(RichText::new("Latest").size(12.5).color(DIM));
            ui.label(RichText::new(latest_tag.clone().unwrap_or_else(|| "—".into())).color(TEXT));
            ui.end_row();
        });

        if let Some((f, text)) = &busy {
            ui.add_space(4.0);
            ui.add(egui::ProgressBar::new(*f).desired_width(right_w).fill(LIGHT).text(RichText::new(text).size(12.0).color(Color32::from_rgb(40, 40, 40))));
        } else if !vers.is_empty() {
            let sel_text = chosen.clone().unwrap_or_default();
            egui::ComboBox::from_id_salt(("vers", i)).width(right_w).selected_text(version_label(&vers, &sel_text, &inst)).show_ui(ui, |ui| {
                for r in &vers {
                    let mut c = app.states[i].chosen.clone().or(latest_tag.clone()).unwrap_or_default();
                    if ui.selectable_value(&mut c, r.tag_name.clone(), version_label(&vers, &r.tag_name, &inst)).clicked() {
                        app.states[i].chosen = Some(c);
                    }
                }
            });
            ui.horizontal(|ui| {
                if let Some(sel) = vers.iter().find(|r| r.tag_name == sel_text) {
                    let (label, is_primary) = match &inst {
                        None if !on_disk => (format!("Install {}", sel.tag_name), true),
                        None => (format!("Install {}", sel.tag_name), false),
                        Some(v) => match cmp_version(&sel.tag_name, v) {
                            std::cmp::Ordering::Greater => (format!("Update to {}", sel.tag_name), true),
                            std::cmp::Ordering::Equal => ("Reinstall".to_string(), false),
                            std::cmp::Ordering::Less => (format!("Downgrade to {}", sel.tag_name), false),
                        },
                    };
                    let clicked = if is_primary { primary(ui, &label).clicked() } else { secondary(ui, &label).clicked() };
                    if clicked {
                        acts.push(Action::Install(i, sel.clone()));
                    }
                }
                if on_disk && secondary(ui, "Remove").on_hover_text("Delete the plugin from the install folder").clicked() {
                    acts.push(Action::Uninstall(i));
                }
            });
        } else if let Some(e) = &err {
            ui.label(RichText::new(e).size(12.0).color(DIM));
        }
    });
}

/// Release notes from the GitHub release bodies (collapsible).
fn notes(app: &mut App, ui: &mut Ui, i: usize) {
    let status = app.status(i);
    let inst = app.installed_version(i);
    let vers: Vec<Release> = app.versions(i).into_iter().cloned().collect();
    if vers.is_empty() {
        return;
    }
    let latest_tag = vers.first().map(|r| r.tag_name.clone());
    let sel = app.states[i].chosen.clone().filter(|t| vers.iter().any(|r| &r.tag_name == t)).or(latest_tag).unwrap_or_default();
    ui.add_space(4.0);
    let newer: Vec<&Release> = match (&inst, status) {
        (Some(v), Status::Update) => vers.iter().filter(|r| cmp_version(&r.tag_name, v).is_gt() && cmp_version(&r.tag_name, &sel).is_le()).collect(),
        _ => vers.iter().filter(|r| r.tag_name == sel).collect(),
    };
    let title = match (&inst, status) {
        (Some(v), Status::Update) => format!("What's new since {v}"),
        _ => format!("Release notes · {sel}"),
    };
    egui::CollapsingHeader::new(RichText::new(title).size(13.5).color(TEXT)).id_salt(("notes", i)).default_open(status == Status::Update).show(ui, |ui| {
        egui::ScrollArea::vertical().id_salt(("notes_scroll", i)).max_height(260.0).show(ui, |ui| {
            ui.set_width(ui.available_width());
            for r in newer {
                release_block(ui, r);
            }
        });
    });
}

fn release_block(ui: &mut Ui, r: &Release) {
    ui.label(RichText::new(format!("{}   {}", r.tag_name, r.date())).strong().size(14.5).color(WHITE));
    match r.body.as_deref().map(str::trim) {
        Some(b) if !b.is_empty() => markdown(ui, b),
        _ => {
            ui.label(RichText::new("No release notes.").color(DIM));
        }
    }
    ui.add_space(8.0);
}

// ------------------------------------------------------------------ plugin page

fn detail_page(app: &mut App, ui: &mut Ui, i: usize, acts: &mut Vec<Action>) {
    let def = &plugins()[i];
    if secondary(ui, "‹  Back").clicked() {
        app.page = Page::All;
    }
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        ui.label(RichText::new(def.name).size(30.0).strong().color(WHITE));
        ui.label(RichText::new(def.tagline).size(15.0).color(DIM));
    });
    ui.horizontal(|ui| {
        for t in def.tags {
            badge(ui, t, false);
        }
        ui.hyperlink_to(RichText::new("GitHub").size(12.5), format!("https://github.com/{}", def.repo));
        ui.hyperlink_to(RichText::new("All releases").size(12.5), format!("https://github.com/{}/releases", def.repo));
    });
    ui.add_space(6.0);

    let right_w = 262.0;
    let left_w = (ui.available_width() - right_w - 20.0).max(300.0);
    ui.horizontal_top(|ui| {
        // ---- gallery
        ui.allocate_ui_with_layout(egui::vec2(left_w, 0.0), Layout::top_down(Align::Min), |ui| {
            let n = def.shots.len();
            app.shot = app.shot.min(n.saturating_sub(1));
            if let Some(s) = def.shots.get(app.shot) {
                Frame::new().fill(Color32::from_rgb(9, 9, 9)).stroke(Stroke::new(1.0, LINE)).corner_radius(10).inner_margin(Margin::same(8)).show(ui, |ui| {
                    let w = ui.available_width();
                    ui.vertical_centered(|ui| {
                        picture(app, ui, &format!("{}:{}", def.id, app.shot), s.data, egui::vec2(w, 430.0));
                    });
                });
                ui.horizontal(|ui| {
                    ui.label(RichText::new(format!("{}  ·  {} / {}", s.caption, app.shot + 1, n)).size(12.5).color(DIM));
                    if n > 1 {
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            if secondary(ui, "›").clicked() {
                                app.shot = (app.shot + 1) % n;
                            }
                            if secondary(ui, "‹").clicked() {
                                app.shot = (app.shot + n - 1) % n;
                            }
                        });
                    }
                });
                if n > 1 {
                    ui.horizontal_wrapped(|ui| {
                        for (k, sh) in def.shots.iter().enumerate() {
                            if let Some((id, size)) = get_tex(app, ui.ctx(), &format!("{}:{}", def.id, k), sh.data) {
                                let h = 54.0;
                                let r = ui.add(egui::Image::new((id, egui::vec2(size.x / size.y * h, h))).corner_radius(4).sense(egui::Sense::click()));
                                if k == app.shot {
                                    ui.painter().rect_stroke(r.rect.expand(2.0), 5.0, Stroke::new(2.0, LIGHT), egui::StrokeKind::Outside);
                                }
                                if r.on_hover_cursor(egui::CursorIcon::PointingHand).on_hover_text(sh.caption).clicked() {
                                    app.shot = k;
                                }
                            }
                        }
                    });
                }
            }
        });
        // ---- install controls
        controls(app, ui, i, acts, right_w);
    });

    ui.add_space(14.0);
    section(ui, "About", |ui| {
        ui.label(RichText::new(def.long).size(14.0).color(Color32::from_rgb(190, 190, 190)));
        ui.add_space(6.0);
        for f in def.features {
            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing.x = 0.0;
                ui.label(RichText::new("•  ").color(DIM));
                ui.label(RichText::new(*f).color(TEXT));
            });
        }
    });

    section(ui, "Release history", |ui| {
        let vers: Vec<Release> = app.versions(i).into_iter().cloned().collect();
        if vers.is_empty() {
            ui.label(RichText::new("No releases found yet. Press “Check for updates”.").color(DIM));
        }
        for (k, r) in vers.iter().enumerate() {
            egui::CollapsingHeader::new(RichText::new(format!("{}   {}", r.tag_name, r.date())).size(14.0).color(TEXT))
                .id_salt(("hist", i, k))
                .default_open(k == 0)
                .show(ui, |ui| release_block(ui, r));
        }
    });
}

fn version_label(vers: &[Release], tag: &str, installed: &Option<String>) -> String {
    let mut s = tag.to_string();
    if let Some(r) = vers.iter().find(|r| r.tag_name == tag) {
        if !r.date().is_empty() {
            s += &format!("  ·  {}", r.date());
        }
        if r.prerelease {
            s += "  ·  pre-release";
        }
    }
    if vers.first().map(|r| r.tag_name.as_str()) == Some(tag) {
        s += "  ·  latest";
    }
    if installed.as_deref() == Some(tag) {
        s += "  ·  installed";
    }
    s
}

// ------------------------------------------------------------------ settings

fn section(ui: &mut Ui, title: &str, add: impl FnOnce(&mut Ui)) {
    Frame::new().fill(CARD).stroke(Stroke::new(1.0, LINE)).corner_radius(10).inner_margin(Margin::same(16)).show(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.label(RichText::new(title).size(16.0).strong().color(WHITE));
        ui.add_space(4.0);
        add(ui);
    });
    ui.add_space(8.0);
}

fn settings_page(app: &mut App, ui: &mut Ui) {
    let before = app.settings.clone();
    ui.label(RichText::new("SETTINGS").size(12.0).color(DIM));
    ui.add_space(2.0);

    section(ui, "Plugin folder", |ui| {
        ui.label(RichText::new("Where the .vst3 bundles are installed. Your DAW must scan this folder.").size(13.0).color(DIM));
        ui.horizontal(|ui| {
            ui.add(egui::TextEdit::singleline(&mut app.settings.install_dir).desired_width(ui.available_width() - 250.0));
            if secondary(ui, "Browse…").clicked() {
                if let Some(p) = rfd::FileDialog::new().set_directory(app.settings.dir()).pick_folder() {
                    app.settings.install_dir = p.to_string_lossy().into_owned();
                }
            }
            if secondary(ui, "Default").clicked() {
                app.settings.install_dir = default_install_dir().to_string_lossy().into_owned();
            }
            if secondary(ui, "Open").clicked() {
                let d = app.settings.dir();
                let _ = std::fs::create_dir_all(&d);
                open_path(&d);
            }
        });
        let d = app.settings.dir();
        ui.label(RichText::new(if d.exists() { "Folder exists" } else { "Folder does not exist yet. It will be created on the first install." }).size(12.5).color(DIM));
        let hint = match OS {
            Os::Windows => "Tip: the default C:\\Program Files\\Common Files\\VST3 needs administrator rights. For a user folder choose e.g. %LOCALAPPDATA%\\Programs\\Common\\VST3.",
            Os::MacOs => "Tip: ~/Library/Audio/Plug-Ins/VST3 is read by every DAW without admin rights.",
            Os::Linux => "Tip: ~/.vst3 is read by most DAWs. /usr/lib/vst3 is system-wide and needs root.",
        };
        ui.label(RichText::new(hint).size(12.5).color(DIM));
    });

    section(ui, "Updates", |ui| {
        ui.checkbox(&mut app.settings.check_on_startup, "Check for updates when the launcher starts");
        ui.horizontal(|ui| {
            ui.label("Re-check in the background every");
            ui.add(egui::DragValue::new(&mut app.settings.background_minutes).range(0..=1440).suffix(" min"));
            ui.label(RichText::new("(0 = off, works while the launcher is open)").size(12.5).color(DIM));
        });
        ui.checkbox(&mut app.settings.notify, "Show a desktop notification when a new version appears");
        ui.checkbox(&mut app.settings.include_prereleases, "Include pre-release versions");
        ui.checkbox(&mut app.settings.keep_downloads, "Keep downloaded .zip files after installing");
        ui.add_space(4.0);
        ui.label(RichText::new("GitHub token (optional)").size(13.0).color(TEXT));
        ui.add(egui::TextEdit::singleline(&mut app.settings.github_token).password(true).desired_width(420.0).hint_text("only needed if you hit the GitHub rate limit"));
        ui.label(RichText::new("Unchanged repositories are checked with ETags, which cost no rate limit. The token is stored in plain text in the settings file.").size(12.5).color(DIM));
    });

    section(ui, "Storage", |ui| {
        let cache = cache_dir();
        ui.horizontal(|ui| {
            ui.label(RichText::new(format!("Cache (release lists, downloads): {}", human_size(dir_size(&cache)))).color(TEXT));
            if secondary(ui, "Clear cache").clicked() {
                let _ = std::fs::remove_dir_all(&cache);
                let _ = std::fs::create_dir_all(&cache);
                app.status = "Cache cleared".into();
            }
        });
        ui.label(RichText::new(format!("Settings: {}", config_dir().display())).size(12.5).color(DIM));
        if secondary(ui, "Open settings folder").clicked() {
            open_path(&config_dir());
        }
    });

    section(ui, "About", |ui| {
        ui.label(RichText::new(format!("Nyashk Launcher v{}", env!("CARGO_PKG_VERSION"))).color(TEXT));
        ui.label(RichText::new("Plugins, versions and release notes come straight from GitHub Releases.").size(13.0).color(DIM));
    });

    if app.settings != before {
        app.settings.save();
        if app.settings.install_dir != before.install_dir {
            app.refresh_disk();
        }
    }
}
