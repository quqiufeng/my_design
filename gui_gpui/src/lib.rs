#![allow(dead_code)]
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int, c_void};
use std::sync::{Arc, Mutex};

use gpui::*;
use serde_json::Value;

mod widgets;
use widgets::{h_flex, v_flex};

// ── Color palette ─────────────────────────────────────────────

mod palette {
    use gpui::Rgba;
    pub fn bg_deep() -> Rgba { Rgba { r: 0.96, g: 0.97, b: 0.98, a: 1.0 } }
    pub fn bg_surface() -> Rgba { Rgba { r: 1.0, g: 1.0, b: 1.0, a: 1.0 } }
    pub fn bg_card() -> Rgba { Rgba { r: 0.94, g: 0.95, b: 0.96, a: 1.0 } }
    pub fn bg_elevated() -> Rgba { Rgba { r: 0.90, g: 0.91, b: 0.93, a: 1.0 } }
    pub fn border() -> Rgba { Rgba { r: 0.87, g: 0.88, b: 0.90, a: 1.0 } }
    pub fn border_light() -> Rgba { Rgba { r: 0.92, g: 0.93, b: 0.94, a: 1.0 } }
    pub fn text_primary() -> Rgba { Rgba { r: 0.10, g: 0.10, b: 0.15, a: 1.0 } }
    pub fn text_secondary() -> Rgba { Rgba { r: 0.42, g: 0.44, b: 0.50, a: 1.0 } }
    pub fn text_muted() -> Rgba { Rgba { r: 0.60, g: 0.62, b: 0.68, a: 1.0 } }
    pub fn accent() -> Rgba { Rgba { r: 0.39, g: 0.38, b: 0.95, a: 1.0 } }
    pub fn accent_hover() -> Rgba { Rgba { r: 0.49, g: 0.48, b: 0.98, a: 1.0 } }
    pub fn accent_dim() -> Rgba { Rgba { r: 0.90, g: 0.89, b: 0.99, a: 1.0 } }
    pub fn green() -> Rgba { Rgba { r: 0.13, g: 0.77, b: 0.37, a: 1.0 } }
    pub fn orange() -> Rgba { Rgba { r: 0.96, g: 0.62, b: 0.04, a: 1.0 } }
    pub fn pink() -> Rgba { Rgba { r: 0.94, g: 0.38, b: 0.62, a: 1.0 } }
    pub fn cyan() -> Rgba { Rgba { r: 0.04, g: 0.69, b: 0.81, a: 1.0 } }
}

fn font_stack() -> SharedString {
    SharedString::from("Inter, -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif")
}

// ── Message ──────────────────────────────────────────────────

#[derive(Clone)]
struct MessageRow {
    role: String,
    text: String,
    is_design: bool,
}

// ── GuiApp ────────────────────────────────────────────────────

pub struct GuiApp {
    on_user_message: Option<extern "C" fn(*const c_char, *const c_char, *mut c_void)>,
    user_data: *mut c_void,
    messages: Arc<Mutex<Vec<MessageRow>>>,
    lua_state: *mut c_void,
}

unsafe impl Send for GuiApp {}
unsafe impl Sync for GuiApp {}

// ── C ABI ─────────────────────────────────────────────────────

#[no_mangle]
pub extern "C" fn gui_app_create(config_json: *const c_char) -> *mut c_void {
    if config_json.is_null() { return std::ptr::null_mut(); }
    let config_str = unsafe { CStr::from_ptr(config_json).to_string_lossy() };
    let _ = match serde_json::from_str::<Value>(&config_str) {
        Ok(Value::Object(m)) => m,
        _ => serde_json::Map::new(),
    };
    let app = GuiApp {
        on_user_message: None,
        user_data: std::ptr::null_mut(),
        messages: Arc::new(Mutex::new(Vec::new())),
        lua_state: std::ptr::null_mut(),
    };
    Box::into_raw(Box::new(app)) as *mut c_void
}

#[no_mangle]
pub extern "C" fn gui_app_free(app: *mut c_void) {
    if !app.is_null() { unsafe { drop(Box::from_raw(app as *mut GuiApp)) }; }
}

#[no_mangle]
pub extern "C" fn gui_on_user_message(
    app: *mut c_void, callback: extern "C" fn(*const c_char, *const c_char, *mut c_void), userdata: *mut c_void,
) {
    if app.is_null() { return; }
    let app: &mut GuiApp = unsafe { &mut *(app as *mut GuiApp) };
    app.on_user_message = Some(callback);
    app.user_data = userdata;
}

#[no_mangle]
pub extern "C" fn gui_stream_delta(app: *mut c_void, _session_id: *const c_char, delta: *const c_char) {
    if app.is_null() || delta.is_null() { return; }
    let delta_str = unsafe { CStr::from_ptr(delta).to_string_lossy().to_string() };
    let app: &GuiApp = unsafe { &*(app as *mut GuiApp) };
    let mut msgs = app.messages.lock().unwrap();
    if let Some(last) = msgs.last_mut() {
        if last.role == "assistant" { last.text.push_str(&delta_str); return; }
    }
    msgs.push(MessageRow { role: "assistant".to_string(), text: delta_str, is_design: false });
}

#[no_mangle]
pub extern "C" fn gui_append_message(app: *mut c_void, _session_id: *const c_char, role: *const c_char, text: *const c_char) {
    if app.is_null() || role.is_null() || text.is_null() { return; }
    let role_str = unsafe { CStr::from_ptr(role).to_string_lossy().to_string() };
    let text_str = unsafe { CStr::from_ptr(text).to_string_lossy().to_string() };
    let is_design = role_str == "design";
    let app: &GuiApp = unsafe { &*(app as *mut GuiApp) };
    app.messages.lock().unwrap().push(MessageRow { role: role_str, text: text_str, is_design });
}

#[no_mangle]
pub extern "C" fn gui_run(app_ptr: *mut c_void, lua_state: *mut c_void) -> c_int {
    if app_ptr.is_null() { return -1; }
    {
        let app: &mut GuiApp = unsafe { &mut *(app_ptr as *mut GuiApp) };
        app.lua_state = lua_state;
    }
    let app_ptr = app_ptr as *mut GuiApp;

    gpui_platform::application().run(move |cx: &mut App| {
        cx.activate(true);
        let bounds = Bounds::centered(None, size(px(1400.0), px(860.0)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                #[cfg(target_os = "linux")]
                window_background: gpui::WindowBackgroundAppearance::Opaque,
                window_min_size: Some(gpui::Size { width: px(900.), height: px(600.) }),
                titlebar: Some(gpui::TitlebarOptions {
                    title: Some(gpui::SharedString::from("my_design - AI UI Designer")),
                    ..Default::default()
                }),
                ..Default::default()
            },
            move |_window, cx| {
                cx.new(|_| DesignView {
                    app: app_ptr,
                    input_text: String::new(),
                })
            },
        ).ok();
    });
    0
}

// ── DesignView ────────────────────────────────────────────────

struct DesignView {
    app: *mut GuiApp,
    input_text: String,
}

const SKILLS: &[(&str, &str, &str, &str)] = &[
    ("web-landing",  "🌐 Web Landing" , "Web Landing", "web"),
    ("mobile-ios",   "📱 iOS App"     , "iOS App", "mobile"),
    ("mobile-md",    "🟣 Material"    , "Material", "mobile"),
    ("dashboard",    "📊 Dashboard"   , "Dashboard", "web"),
    ("ecommerce",    "🛍️ E-Commerce" , "E-Commerce", "web"),
    ("settings",     "⚙️ Settings"    , "Settings", "mobile"),
];

impl DesignView {
    fn send_message(&mut self, cx: &mut Context<Self>) {
        let text = self.input_text.trim().to_string();
        if text.is_empty() { return; }
        let app = self.app;
        {
            let app_ref: &mut GuiApp = unsafe { &mut *app };
            if let Some(cb) = app_ref.on_user_message {
                let s = CString::new("default").unwrap();
                let t = CString::new(text).unwrap();
                cb(s.as_ptr(), t.as_ptr(), app_ref.user_data);
            }
        }
        self.input_text.clear();
        cx.notify();
    }

    fn handle_key(&mut self, event: &KeyDownEvent, cx: &mut Context<Self>) {
        // Use key_char for composed characters, otherwise fall back to keystroke.key
        let key_str = event.keystroke.key_char.as_ref()
            .unwrap_or(&event.keystroke.key);

        match key_str.as_str() {
            "backspace" | "delete" => {
                self.input_text.pop();
                cx.notify();
            }
            "enter" | "return" => {
                if !event.keystroke.modifiers.shift {
                    self.send_message(cx);
                } else {
                    self.input_text.push('\n');
                    cx.notify();
                }
            }
            "space" => {
                self.input_text.push(' ');
                cx.notify();
            }
            // Skip control keys
            s if s.len() == 1 => {
                self.input_text.push_str(s);
                cx.notify();
            }
            _ => {}
        }
    }
}

impl Render for DesignView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let bg = palette::bg_deep();
        let surface = palette::bg_surface();
        let card = palette::bg_card();
        let border = palette::border();
        let text_pri = palette::text_primary();
        let text_sec = palette::text_secondary();
        let accent = palette::accent();

        let messages = unsafe { &*self.app }.messages.lock().unwrap().clone();
        let input_display = if self.input_text.is_empty() {
            SharedString::from("Describe the UI you want to create...")
        } else {
            SharedString::from(self.input_text.as_str())
        };
        let input_color = if self.input_text.is_empty() { text_sec } else { text_pri };

        // ── Left panel ──
        let left_panel = v_flex()
            .w(px(220.0)).h_full().bg(surface).border_r_1().border_color(border)
            .child(
                h_flex().px_4().py(px(16.0)).gap_2()
                    .child(div().w(px(28.0)).h(px(28.0)).rounded_lg().bg(accent)
                        .flex().items_center().justify_center()
                        .child(div().child("M").text_color(gpui::Rgba { r: 1.0, g: 1.0, b: 1.0, a: 1.0 })
                            .font_weight(FontWeight::BOLD).text_base().font_family(font_stack())))
                    .child(div().child("my_design").text_color(text_pri).text_base().font_weight(FontWeight::SEMIBOLD).font_family(font_stack())))
            .child(v_flex().flex_1().px_3()
                .child(div().px_1().py(px(8.0)).text_xs().font_weight(FontWeight::BOLD).text_color(text_sec).font_family(font_stack())
                    .child("STYLES".to_uppercase()))
                .children(SKILLS.iter().enumerate().map(|(i, (_id, name, icon, _cat))| {
                    let is_first = i == 0;
                    div().px_3().py(px(8.0)).rounded_md()
                        .bg(if is_first { palette::accent_dim() } else { gpui::Rgba { r: 0.0, g: 0.0, b: 0.0, a: 0.0 } })
                        .hover(|s| s.bg(if is_first { palette::accent_dim() } else { palette::bg_elevated() }))
                        .cursor_pointer()
                        .child(h_flex().gap_3().items_center()
                            .child(div().child(icon.to_string()).text_base())
                            .child(div().child(name.to_string()).text_color(text_pri).text_sm().font_family(font_stack())))
                })))
            .child(div().px_4().py(px(12.0)).border_t_1().border_color(border)
                .child(div().child("AI UI Designer v0.1").text_color(text_sec).text_xs().font_family(font_stack())));

        // ── Canvas area ──
        let canvas = v_flex().flex_1().h_full().bg(bg).overflow_hidden()
            .child(v_flex().flex_1().px_6().py(px(16.0)).gap_4()
                .children(if messages.is_empty() {
                    vec![v_flex().flex_1().items_center().justify_center().gap_4()
                        .child(div().w(px(80.0)).h(px(80.0)).rounded_2xl()
                            .bg(gpui::Rgba { r: 0.39, g: 0.38, b: 0.95, a: 0.15 })
                            .flex().items_center().justify_center()
                            .child(div().child("✨").text_2xl()))
                        .child(div().child("What would you like to design today?")
                            .text_color(text_pri).text_xl().font_weight(FontWeight::SEMIBOLD).font_family(font_stack()))
                        .child(div().child("Type your idea below and click Generate — AI will create a draft based on the selected style. You can then refine it.")
                            .text_color(text_sec).text_base().font_family(font_stack()))
                        .into_any_element()]
                } else {
                    messages.iter().enumerate().map(|(idx, m)| {
                        let (label, ac) = match m.role.as_str() {
                            "user" => ("You", palette::cyan()),
                            "assistant" => ("AI", palette::accent()),
                            "design" => ("💎 Design", palette::pink()),
                            _ => ("System", palette::text_muted()),
                        };
                        let msg_bg = if idx % 2 == 0 { card } else { gpui::Rgba { r: 0.0, g: 0.0, b: 0.0, a: 0.0 } };

                        v_flex().bg(msg_bg).rounded_lg().p_4().gap_2()
                            .child(h_flex().gap_2().items_center()
                                .child(div().w(px(6.0)).h(px(6.0)).rounded_full().bg(ac))
                                .child(div().child(label).text_color(ac).text_sm().font_weight(FontWeight::SEMIBOLD).font_family(font_stack())))
                            .child(if m.is_design {
                                // Design preview card
                                v_flex().gap_2()
                                    .child(div().child(m.text.clone()).text_color(text_pri).text_base().font_family(font_stack()))
                                    .child(
                                        div().px_4().py(px(24.0)).rounded_lg()
                                            .bg(gpui::Rgba { r: 0.39, g: 0.38, b: 0.95, a: 0.06 })
                                            .border_1().border_color(palette::accent_dim())
                                            .flex().items_center().justify_center()
                                            .child(h_flex().gap_3().items_center()
                                                .child(div().child("🖼").text_lg())
                                                .child(div().child("Design Preview").text_color(text_sec).text_sm().font_family(font_stack())))
                                    )
                                    .into_any_element()
                            } else {
                                div().child(m.text.clone()).text_color(text_pri).text_base().font_family(font_stack())
                                    .into_any_element()
                            })
                            .into_any_element()
                    }).collect()
                }))
            // ── Input bar ──
            .child(v_flex().px_6().py(px(16.0)).border_t_1().border_color(border).bg(surface).gap_3()
                .child(h_flex().gap_2()
                    .child(chip("Web", true, accent))
                    .child(chip("Mobile", false, palette::bg_elevated()))
                    .child(chip("Dashboard", false, palette::bg_elevated())))
                .child(h_flex().gap_3()
                    .child(
                        // ═══ KEYBOARD INPUT ═══
                        div().flex_1()
                            .id("prompt-input")
                            .px_4().py(px(12.0)).rounded_xl()
                            .bg(palette::bg_card()).border_1().border_color(border)
                            .cursor_text()
                            .child(div().child(input_display.to_string()).text_color(input_color).text_base().font_family(font_stack()))
                            .on_key_down(cx.listener(|this, event, _window, _cx| {
                                this.handle_key(event, _cx);
                            }))
                            .on_mouse_down(gpui::MouseButton::Left, |_event, _window, _cx| {
                                // Click handled, focus is automatic with .id()
                            })
                    )
                    .child(
                        // ═══ GENERATE BUTTON ═══
                        div().px_5().py(px(12.0)).rounded_xl().bg(accent)
                            .hover(|s| s.bg(palette::accent_hover()))
                            .cursor_pointer()
                            .on_mouse_down(gpui::MouseButton::Left, cx.listener(|this, _event, _window, cx| {
                                this.send_message(cx);
                            }))
                            .child(h_flex().gap_2().items_center()
                                .child(div().child("✨").text_sm())
                                .child(div().child("Generate").text_color(gpui::Rgba { r: 1.0, g: 1.0, b: 1.0, a: 1.0 })
                                    .text_sm().font_weight(FontWeight::SEMIBOLD).font_family(font_stack())))
                    )));

        // ── Right panel ──
        let right_panel = v_flex().w(px(260.0)).h_full().bg(surface).border_l_1().border_color(border)
            .child(v_flex().px_4().py(px(16.0)).gap_4()
                .child(v_flex().gap_3()
                    .child(section_title("DESIGN TOKENS"))
                    .child(token_row("Primary",   "#6366f1", palette::accent()))
                    .child(token_row("Secondary", "#8b5cf6", palette::accent_hover()))
                    .child(token_row("Background","#ffffff", gpui::Rgba { r: 1.0, g: 1.0, b: 1.0, a: 1.0 }))
                    .child(token_row("Text",      "#1e1e1e", gpui::Rgba { r: 0.12, g: 0.12, b: 0.12, a: 1.0 }))
                    .child(token_row("Radius",    "12px",    gpui::Rgba { r: 0.0, g: 0.0, b: 0.0, a: 0.0 })))
                .child(divider(border))
                .child(v_flex().gap_3()
                    .child(section_title("EXPORT"))
                    .child(export_btn("React Component", palette::cyan()))
                    .child(export_btn("HTML + CSS",     palette::green()))
                    .child(export_btn("Figma (JSON)",   palette::orange())))
                .child(divider(border))
                .child(v_flex().gap_2()
                    .child(section_title("RECENT"))
                    .child(history_item("Landing page v3", "2 min ago", palette::accent()))
                    .child(history_item("Login screen", "15 min ago", palette::cyan()))
                    .child(history_item("Dashboard", "1 hour ago", palette::green()))));

        h_flex().size_full().bg(bg)
            .child(left_panel)
            .child(canvas)
            .child(right_panel)
    }
}

// ── Helpers ───────────────────────────────────────────────────

fn section_title(text: &str) -> impl IntoElement {
    div().child(text.to_uppercase()).text_xs().font_weight(FontWeight::BOLD)
        .text_color(palette::text_secondary()).font_family(font_stack())
}

fn chip(text: &str, active: bool, _bg: Rgba) -> impl IntoElement {
    let t = SharedString::from(text);
    let bg = if active { palette::accent() } else { palette::bg_elevated() };
    let tc = if active { gpui::Rgba { r: 1.0, g: 1.0, b: 1.0, a: 1.0 } } else { palette::text_secondary() };
    div().px_3().py(px(4.0)).rounded_full().bg(bg)
        .child(div().child(t).text_color(tc).text_xs().font_weight(FontWeight::MEDIUM).font_family(font_stack()))
}

fn divider(border: Rgba) -> impl IntoElement {
    div().h(px(1.0)).w_full().bg(border)
}

fn token_row(label: &str, value: &str, color: Rgba) -> AnyElement {
    let text_pri = palette::text_primary();
    let text_sec = palette::text_secondary();
    let l = SharedString::from(label);
    let v = SharedString::from(value);
    h_flex().gap_3().items_center()
        .child(div().w(px(16.0)).h(px(16.0)).rounded_md().bg(color).border_1()
            .border_color(if color.r > 0.9 && color.g > 0.9 && color.b > 0.9 { palette::border_light() } else { gpui::Rgba { r: 0.0, g: 0.0, b: 0.0, a: 0.0 } }))
        .child(div().child(l).text_color(text_sec).text_sm().font_family(font_stack()).flex_1())
        .child(div().child(v).text_color(text_pri).text_sm().font_family(mono_font()))
        .into_any_element()
}

fn mono_font() -> SharedString {
    SharedString::from("'SF Mono', 'Fira Code', 'Cascadia Code', monospace")
}

fn export_btn(label: &str, accent: Rgba) -> AnyElement {
    let text_pri = palette::text_primary();
    let l = SharedString::from(label);
    h_flex().px_3().py(px(8.0)).rounded_lg().bg(palette::bg_card())
        .hover(|s| s.bg(palette::bg_elevated())).cursor_pointer()
        .gap_2().items_center()
        .child(div().w(px(8.0)).h(px(8.0)).rounded_full().bg(accent))
        .child(div().child(l).text_color(text_pri).text_sm().font_family(font_stack()))
        .into_any_element()
}

fn history_item(label: &str, time: &str, dot: Rgba) -> AnyElement {
    let text_pri = palette::text_primary();
    let text_sec = palette::text_secondary();
    let l = SharedString::from(label);
    let t = SharedString::from(time);
    h_flex().px_3().py(px(6.0)).rounded_md()
        .hover(|s| s.bg(palette::bg_card())).cursor_pointer()
        .gap_2().items_center()
        .child(div().w(px(6.0)).h(px(6.0)).rounded_full().bg(dot))
        .child(div().child(l).text_color(text_pri).text_sm().font_family(font_stack()).flex_1())
        .child(div().child(t).text_color(text_sec).text_xs().font_family(font_stack()))
        .into_any_element()
}

#[no_mangle]
pub extern "C" fn gui_refresh(_app: *mut c_void) {}
