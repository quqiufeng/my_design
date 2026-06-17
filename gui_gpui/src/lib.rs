#![allow(dead_code)]
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int, c_void};
use std::sync::{Arc, Mutex};

use gpui::*;

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
    pub fn cyan() -> Rgba { Rgba { r: 0.04, g: 0.69, b: 0.81, a: 1.0 } }
}

fn font_stack() -> SharedString {
    SharedString::from("Inter, -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif")
}

// ── Page item ─────────────────────────────────────────────────

#[derive(Clone, Debug)]
struct PageItem {
    name: String,
    selected: bool,
    done: bool,
}

// ── App stage ─────────────────────────────────────────────────

#[derive(Clone, Debug, PartialEq)]
enum AppStage {
    Input,
    Splitting,
    Confirm,
    Generating,
}

// ── GuiApp (thread-safe shared state) ─────────────────────────

pub struct GuiApp {
    on_user_message: Option<extern "C" fn(*const c_char, *const c_char, *mut c_void)>,
    user_data: *mut c_void,
    pages: Arc<Mutex<Vec<PageItem>>>,
    lua_state: *mut c_void,
}

unsafe impl Send for GuiApp {}
unsafe impl Sync for GuiApp {}

// ── C ABI ─────────────────────────────────────────────────────

#[no_mangle]
pub extern "C" fn gui_app_create(_config_json: *const c_char) -> *mut c_void {
    let app = GuiApp {
        on_user_message: None,
        user_data: std::ptr::null_mut(),
        pages: Arc::new(Mutex::new(Vec::new())),
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

/// Lua calls this to set the page list after AI splitting
#[no_mangle]
pub extern "C" fn gui_set_pages(app: *mut c_void, pages_json: *const c_char) {
    if app.is_null() || pages_json.is_null() { return; }
    let json_str = unsafe { CStr::from_ptr(pages_json).to_string_lossy() };
    if let Ok(pages) = serde_json::from_str::<Vec<String>>(&json_str) {
        let app: &GuiApp = unsafe { &*(app as *mut GuiApp) };
        let mut list = app.pages.lock().unwrap();
        list.clear();
        for name in pages {
            list.push(PageItem { name, selected: true, done: false });
        }
    }
}

/// Lua calls this to mark a page as done
#[no_mangle]
pub extern "C" fn gui_set_page_done(app: *mut c_void, index: c_int) {
    if app.is_null() { return; }
    let app: &GuiApp = unsafe { &*(app as *mut GuiApp) };
    let mut list = app.pages.lock().unwrap();
    if index >= 0 && (index as usize) < list.len() {
        list[index as usize].done = true;
    }
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
                    stage: AppStage::Input,
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
    stage: AppStage,
}

impl DesignView {
    fn send_to_lua(&mut self, text: &str, cx: &mut Context<Self>) {
        if text.trim().is_empty() { return; }
        let app = self.app;
        {
            let app_ref: &mut GuiApp = unsafe { &mut *app };
            if let Some(cb) = app_ref.on_user_message {
                let s = CString::new("default").unwrap();
                let t = CString::new(text).unwrap();
                cb(s.as_ptr(), t.as_ptr(), app_ref.user_data);
            }
        }
        cx.notify();
    }

    fn handle_key(&mut self, event: &KeyDownEvent, cx: &mut Context<Self>) {
        let key_str = event.keystroke.key_char.as_ref()
            .unwrap_or(&event.keystroke.key);
        match key_str.as_str() {
            "backspace" | "delete" => { self.input_text.pop(); cx.notify(); }
            "enter" | "return" => {
                if self.stage == AppStage::Input && !self.input_text.trim().is_empty() {
                    self.start_split(cx);
                }
            }
            "space" => { self.input_text.push(' '); cx.notify(); }
            s if s.len() == 1 => { self.input_text.push_str(s); cx.notify(); }
            _ => {}
        }
    }

    fn start_split(&mut self, cx: &mut Context<Self>) {
        let req = self.input_text.trim().to_string();
        if req.is_empty() { return; }
        self.stage = AppStage::Splitting;
        self.send_to_lua(&format!("/split {}", req), cx);
    }

    fn start_generate(&mut self, cx: &mut Context<Self>) {
        self.stage = AppStage::Generating;
        // Collect selected page names
        let pages = unsafe { &*self.app }.pages.lock().unwrap().clone();
        let names: Vec<String> = pages.iter().filter(|p| p.selected).map(|p| p.name.clone()).collect();
        self.send_to_lua(&format!("/generate {}", serde_json::to_string(&names).unwrap_or_default()), cx);
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

        // ═══ LEFT PANEL ═══
        let left_panel = v_flex()
            .w(px(220.0)).h_full().bg(surface).border_r_1().border_color(border)
            .child(h_flex().px_4().py(px(16.0)).gap_2()
                .child(div().w(px(28.0)).h(px(28.0)).rounded_lg().bg(accent)
                    .flex().items_center().justify_center()
                    .child(div().child("M").text_color(gpui::Rgba { r: 1.0, g: 1.0, b: 1.0, a: 1.0 })
                        .font_weight(FontWeight::BOLD).text_base().font_family(font_stack())))
                .child(div().child("my_design").text_color(text_pri).text_base().font_weight(FontWeight::SEMIBOLD).font_family(font_stack())))
            .child(v_flex().flex_1().px_3().py(px(12.0))
                .child(section_title("PROJECT"))
                .child(div().px_2().py(px(8.0)).rounded_md().bg(card)
                    .child(div().child("New Project").text_color(text_pri).text_sm().font_family(font_stack()))
                    .cursor_pointer().on_mouse_down(gpui::MouseButton::Left, cx.listener(|this, _, _w, cx| {
                        this.input_text.clear();
                        this.stage = AppStage::Input;
                        unsafe { &*this.app }.pages.lock().unwrap().clear();
                        cx.notify();
                    }))))
            .child(div().px_4().py(px(12.0)).border_t_1().border_color(border)
                .child(div().child("v0.2 · Page Split Workflow").text_color(text_sec).text_xs().font_family(font_stack())));

        // ═══ MAIN AREA ═══
        let pages = unsafe { &*self.app }.pages.lock().unwrap().clone();

        let main_area = v_flex().flex_1().h_full().bg(bg)
            .child(v_flex().flex_1().px_6().py(px(20.0)).gap_4()
                .children(match self.stage {
                    AppStage::Input => {
                        vec![v_flex().flex_1().items_center().justify_center().gap_4()
                            .child(div().w(px(80.0)).h(px(80.0)).rounded_2xl()
                                .bg(gpui::Rgba { r: 0.39, g: 0.38, b: 0.95, a: 0.15 })
                                .flex().items_center().justify_center()
                                .child(div().child("🚀").text_2xl()))
                            .child(div().child("What are you building?")
                                .text_color(text_pri).text_xl().font_weight(FontWeight::SEMIBOLD).font_family(font_stack()))
                            .child(div().child("Describe your project — I'll split it into pages for you.")
                                .text_color(text_sec).text_base().font_family(font_stack()))
                            .into_any_element()]
                    }
                    AppStage::Splitting => {
                        vec![v_flex().flex_1().items_center().justify_center().gap_3()
                            .child(div().child("🔍").text_2xl())
                            .child(div().child("Analyzing your requirements...")
                                .text_color(text_pri).text_base().font_family(font_stack()))
                            .child(div().child("AI is figuring out what pages you need.")
                                .text_color(text_sec).text_sm().font_family(font_stack()))
                            .into_any_element()]
                    }
                    AppStage::Confirm => {
                        let all_selected = pages.iter().all(|p| p.selected);
                        let mut items: Vec<AnyElement> = Vec::new();
                        // Header
                        items.push(
                            v_flex().gap_2()
                                .child(h_flex().gap_3().items_center()
                                    .child(div().child("📋").text_lg())
                                    .child(div().child("Suggested Pages").text_color(text_pri).text_lg().font_weight(FontWeight::SEMIBOLD).font_family(font_stack()))
                                    .child(div().child(format!("({})", pages.len())).text_color(text_sec).text_sm().font_family(font_stack())))
                                .child(div().child("Select the pages you want to generate. All pages will share the same style.")
                                    .text_color(text_sec).text_sm().font_family(font_stack()))
                                .into_any_element()
                        );
                        // Select all toggle
                        items.push(
                            h_flex().gap_2().items_center().cursor_pointer()
                                .on_mouse_down(gpui::MouseButton::Left, cx.listener(move |this, _, _w, cx| {
                                    let mut list = unsafe { &*this.app }.pages.lock().unwrap();
                                    let new_val = !all_selected;
                                    for p in list.iter_mut() { p.selected = new_val; }
                                    cx.notify();
                                }))
                                .child(div().w(px(18.0)).h(px(18.0)).rounded_md()
                                    .bg(if all_selected { accent } else { gpui::Rgba { r: 0.0, g: 0.0, b: 0.0, a: 0.0 } })
                                    .border_1().border_color(if all_selected { accent } else { border }))
                                .child(div().child(if all_selected { "Deselect all" } else { "Select all" })
                                    .text_color(text_sec).text_sm().font_family(font_stack()))
                                .into_any_element()
                        );
                        // Page list
                        for (i, p) in pages.iter().enumerate() {
                            let page_names = vec!["🛒 ", "📄 ", "🛵 ", "💳 ", "👤 ", "📦 ", "⚙️ ", "🏠 "];
                            let icon = page_names.get(i).unwrap_or(&"📄 ");
                            let idx = i;
                            let selected = p.selected;
                            items.push(
                                h_flex().px_4().py(px(10.0)).rounded_lg()
                                    .bg(if selected { card } else { gpui::Rgba { r: 0.0, g: 0.0, b: 0.0, a: 0.0 } })
                                    .hover(|s| s.bg(card))
                                    .cursor_pointer().gap_3().items_center()
                                    .on_mouse_down(gpui::MouseButton::Left, cx.listener(move |this, _, _w, cx| {
                                        let mut list = unsafe { &*this.app }.pages.lock().unwrap();
                                        if idx < list.len() { list[idx].selected = !list[idx].selected; }
                                        cx.notify();
                                    }))
                                    .child(div().w(px(18.0)).h(px(18.0)).rounded_md()
                                        .bg(if selected { accent } else { gpui::Rgba { r: 0.0, g: 0.0, b: 0.0, a: 0.0 } })
                                        .border_1().border_color(if selected { accent } else { border })
                                        .flex().items_center().justify_center()
                                        .child(if selected { div().child("✓").text_color(gpui::Rgba { r: 1.0, g: 1.0, b: 1.0, a: 1.0 }).text_xs() } else { div() }))
                                    .child(div().child(*icon).text_base())
                                    .child(div().child(p.name.clone()).text_color(text_pri).text_sm().font_family(font_stack()).flex_1())
                                    .into_any_element()
                            );
                        }
                        // Generate button
                        let has_selected = pages.iter().any(|p| p.selected);
                        items.push(
                            h_flex().gap_3().pt_4()
                                .child(
                                    div().px_6().py(px(12.0)).rounded_xl()
                                        .bg(if has_selected { accent } else { palette::bg_elevated() })
                                        .hover(|s| if has_selected { s.bg(palette::accent_hover()) } else { s })
                                        .cursor_pointer()
                                        .on_mouse_down(gpui::MouseButton::Left, cx.listener(|this, _, _w, cx| {
                                            if unsafe { &*this.app }.pages.lock().unwrap().iter().any(|p| p.selected) {
                                                this.start_generate(cx);
                                            }
                                        }))
                                        .child(h_flex().gap_2().items_center()
                                            .child(div().child("✨").text_sm())
                                            .child(div().child(format!("Generate {} pages", pages.iter().filter(|p| p.selected).count()))
                                                .text_color(gpui::Rgba { r: 1.0, g: 1.0, b: 1.0, a: 1.0 }).text_sm().font_weight(FontWeight::SEMIBOLD).font_family(font_stack())))
                                )
                                .into_any_element()
                        );
                        items
                    }
                    AppStage::Generating => {
                        let done_count = pages.iter().filter(|p| p.done).count();
                        let total = pages.len();
                        let mut items: Vec<AnyElement> = Vec::new();
                        items.push(
                            v_flex().gap_1()
                                .child(h_flex().gap_3().items_center()
                                    .child(div().child("🎨").text_lg())
                                    .child(div().child("Generating Designs").text_color(text_pri).text_lg().font_weight(FontWeight::SEMIBOLD).font_family(font_stack())))
                                .child(div().child(format!("{}/{} pages completed", done_count, total))
                                    .text_color(text_sec).text_sm().font_family(font_stack()))
                                .into_any_element()
                        );
                        for (i, p) in pages.iter().enumerate() {
                            let icons = vec!["🛒", "📄", "🛵", "💳", "👤", "📦", "⚙️", "🏠"];
                            let icon = icons.get(i).unwrap_or(&"📄");
                            let status_icon = if p.done { "✅" } else { "⏳" };
                            let bg_c = if p.done { gpui::Rgba { r: 0.13, g: 0.77, b: 0.37, a: 0.08 } } else { card };
                            items.push(
                                h_flex().px_4().py(px(10.0)).rounded_lg().bg(bg_c).gap_3().items_center()
                                    .child(div().child(status_icon).text_sm())
                                    .child(div().child(*icon).text_base())
                                    .child(div().child(p.name.clone()).text_color(text_pri).text_sm().font_family(font_stack()).flex_1())
                                    .child(if p.done { div().child("Done").text_color(palette::green()).text_xs().font_family(font_stack()) } else { div().child("Generating...").text_color(text_sec).text_xs().font_family(font_stack()) })
                                    .into_any_element()
                            );
                        }
                        items
                    }
                }));

        // ═══ INPUT BAR (visible only in Input stage) ═══
        let input_bar = if self.stage == AppStage::Input {
            let input_display = if self.input_text.is_empty() {
                SharedString::from("e.g. Design a food delivery app...")
            } else {
                SharedString::from(self.input_text.as_str())
            };
            let input_color = if self.input_text.is_empty() { text_sec } else { text_pri };
            let can_split = !self.input_text.trim().is_empty();

            v_flex().px_6().py(px(16.0)).border_t_1().border_color(border).bg(surface).gap_3()
                .child(h_flex().gap_3()
                    .child(div().flex_1().id("prompt-input")
                        .px_4().py(px(12.0)).rounded_xl()
                        .bg(palette::bg_card()).border_1().border_color(border).cursor_text()
                        .child(div().child(input_display.to_string()).text_color(input_color).text_base().font_family(font_stack()))
                        .on_key_down(cx.listener(|this, event, _w, cx| { this.handle_key(event, cx); }))
                        .on_mouse_down(gpui::MouseButton::Left, |_e, _w, _cx| {}))
                    .child(div().px_5().py(px(12.0)).rounded_xl()
                        .bg(if can_split { accent } else { palette::bg_elevated() })
                        .hover(|s| if can_split { s.bg(palette::accent_hover()) } else { s })
                        .cursor_pointer()
                        .on_mouse_down(gpui::MouseButton::Left, cx.listener(|this, _, _w, cx| {
                            if !this.input_text.trim().is_empty() { this.start_split(cx); }
                        }))
                        .child(h_flex().gap_2().items_center()
                            .child(div().child("🔍").text_sm())
                            .child(div().child("Split Requirements")
                                .text_color(gpui::Rgba { r: 1.0, g: 1.0, b: 1.0, a: 1.0 }).text_sm().font_weight(FontWeight::SEMIBOLD).font_family(font_stack()))))
                )
                .into_any_element()
        } else {
            div().into_any_element()
        };

        // ═══ RIGHT PANEL ═══
        let right_panel = v_flex().w(px(260.0)).h_full().bg(surface).border_l_1().border_color(border)
            .child(v_flex().px_4().py(px(16.0)).gap_4()
                .child(v_flex().gap_3()
                    .child(section_title("SKILL"))
                    .child(skill_select("Shadcn/ui", true, accent))
                    .child(skill_select("Material 3", false, palette::bg_elevated()))
                    .child(skill_select("iOS HIG", false, palette::bg_elevated()))
                    .child(skill_select("Ant Design", false, palette::bg_elevated())))
                .child(divider(border))
                .child(v_flex().gap_3()
                    .child(section_title("EXPORT"))
                    .child(export_btn("React", palette::cyan()))
                    .child(export_btn("HTML + CSS", palette::green()))
                    .child(export_btn("Figma", palette::accent())))
                .child(divider(border))
                .child(v_flex().gap_2()
                    .child(section_title("PROJECTS"))
                    .child(history_item("Food Delivery App", "just now", palette::accent()))));

        h_flex().size_full().bg(bg)
            .child(left_panel)
            .child(v_flex().flex_1()
                .child(main_area)
                .child(input_bar))
            .child(right_panel)
    }
}

// ── Helpers ───────────────────────────────────────────────────

fn section_title(text: &str) -> impl IntoElement {
    div().child(text.to_uppercase()).text_xs().font_weight(FontWeight::BOLD)
        .text_color(palette::text_secondary()).font_family(font_stack())
}

fn divider(border: Rgba) -> impl IntoElement {
    div().h(px(1.0)).w_full().bg(border)
}

fn skill_select(text: &str, active: bool, _bg: Rgba) -> impl IntoElement {
    let t = SharedString::from(text);
    let bg = if active { palette::accent_dim() } else { gpui::Rgba { r: 0.0, g: 0.0, b: 0.0, a: 0.0 } };
    let tc = if active { palette::accent() } else { palette::text_secondary() };
    div().px_3().py(px(8.0)).rounded_md().bg(bg)
        .hover(|s| s.bg(if active { palette::accent_dim() } else { palette::bg_elevated() }))
        .cursor_pointer()
        .child(div().child(t).text_color(tc).text_sm().font_family(font_stack()))
}

fn export_btn(label: &str, _accent: Rgba) -> AnyElement {
    let l = SharedString::from(label);
    h_flex().px_3().py(px(8.0)).rounded_lg().bg(palette::bg_card())
        .hover(|s| s.bg(palette::bg_elevated())).cursor_pointer()
        .child(div().child(l).text_color(palette::text_primary()).text_sm().font_family(font_stack()))
        .into_any_element()
}

fn history_item(label: &str, time: &str, dot: Rgba) -> AnyElement {
    let l = SharedString::from(label);
    let t = SharedString::from(time);
    h_flex().px_3().py(px(6.0)).rounded_md()
        .hover(|s| s.bg(palette::bg_card())).cursor_pointer().gap_2().items_center()
        .child(div().w(px(6.0)).h(px(6.0)).rounded_full().bg(dot))
        .child(div().child(l).text_color(palette::text_primary()).text_sm().font_family(font_stack()).flex_1())
        .child(div().child(t).text_color(palette::text_secondary()).text_xs().font_family(font_stack()))
        .into_any_element()
}

#[no_mangle]
pub extern "C" fn gui_refresh(_app: *mut c_void) {}
