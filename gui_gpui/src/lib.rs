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
    pub fn text_primary() -> Rgba { Rgba { r: 0.10, g: 0.10, b: 0.15, a: 1.0 } }
    pub fn text_secondary() -> Rgba { Rgba { r: 0.42, g: 0.44, b: 0.50, a: 1.0 } }
    pub fn accent() -> Rgba { Rgba { r: 0.39, g: 0.38, b: 0.95, a: 1.0 } }
    pub fn accent_hover() -> Rgba { Rgba { r: 0.49, g: 0.48, b: 0.98, a: 1.0 } }
    pub fn accent_dim() -> Rgba { Rgba { r: 0.90, g: 0.89, b: 0.99, a: 1.0 } }
    pub fn green() -> Rgba { Rgba { r: 0.13, g: 0.77, b: 0.37, a: 1.0 } }
}

fn font_stack() -> SharedString {
    SharedString::from("Inter, -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif")
}

// ── Page ──────────────────────────────────────────────────────

#[derive(Clone, Debug)]
struct PageItem {
    name: String,
    selected: bool,
    done: bool,
    prompt: String,
    design_preview: String,
    depth: usize,
}

// ── View state ────────────────────────────────────────────────

#[derive(Clone, PartialEq)]
enum ViewState {
    Input,
    PageList,
    SingleEditor,
    BatchGenerate,
}

// ── GuiApp (thread-safe) ──────────────────────────────────────

pub struct GuiApp {
    on_user_message: Option<extern "C" fn(*const c_char, *const c_char, *mut c_void)>,
    user_data: *mut c_void,
    pages: Arc<Mutex<Vec<PageItem>>>,
}

unsafe impl Send for GuiApp {}
unsafe impl Sync for GuiApp {}

#[no_mangle]
pub extern "C" fn gui_app_create(_: *const c_char) -> *mut c_void {
    Box::into_raw(Box::new(GuiApp {
        on_user_message: None,
        user_data: std::ptr::null_mut(),
        pages: Arc::new(Mutex::new(Vec::new())),
    })) as *mut c_void
}

#[no_mangle]
pub extern "C" fn gui_app_free(app: *mut c_void) {
    if !app.is_null() { unsafe { drop(Box::from_raw(app as *mut GuiApp)) }; }
}

#[no_mangle]
pub extern "C" fn gui_on_user_message(
    app: *mut c_void, cb: extern "C" fn(*const c_char, *const c_char, *mut c_void), ud: *mut c_void,
) {
    if app.is_null() { return; }
    let a: &mut GuiApp = unsafe { &mut *(app as *mut GuiApp) };
    a.on_user_message = Some(cb);
    a.user_data = ud;
}

#[no_mangle]
pub extern "C" fn gui_set_pages(app: *mut c_void, pages_json: *const c_char) {
    if app.is_null() || pages_json.is_null() { return; }
    let json_str = unsafe { CStr::from_ptr(pages_json).to_string_lossy() };
    // Accept either ["name1", "name2"] or [{"name":"...","depth":0},...]
    if let Ok(names) = serde_json::from_str::<Vec<String>>(&json_str) {
        let a: &GuiApp = unsafe { &*(app as *mut GuiApp) };
        let mut list = a.pages.lock().unwrap();
        list.clear();
        for n in names {
            list.push(PageItem { name: n, selected: true, done: false, prompt: String::new(), design_preview: String::new(), depth: 0 });
        }
    } else if let Ok(items) = serde_json::from_str::<Vec<serde_json::Value>>(&json_str) {
        let a: &GuiApp = unsafe { &*(app as *mut GuiApp) };
        let mut list = a.pages.lock().unwrap();
        list.clear();
        for item in items {
            let name = item.get("name").and_then(|v| v.as_str()).unwrap_or("Page").to_string();
            let depth = item.get("depth").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
            list.push(PageItem { name, selected: true, done: false, prompt: String::new(), design_preview: String::new(), depth });
        }
    }
}

#[no_mangle]
pub extern "C" fn gui_set_page_done(app: *mut c_void, index: c_int) {
    if app.is_null() { return; }
    let a: &GuiApp = unsafe { &*(app as *mut GuiApp) };
    let mut list = a.pages.lock().unwrap();
    if index >= 0 && (index as usize) < list.len() {
        list[index as usize].done = true;
    }
}

/// Lua calls this to set a page's design preview content
#[no_mangle]
pub extern "C" fn gui_set_page_preview(app: *mut c_void, index: c_int, text: *const c_char) {
    if app.is_null() || text.is_null() { return; }
    let t = unsafe { CStr::from_ptr(text).to_string_lossy().to_string() };
    let a: &GuiApp = unsafe { &*(app as *mut GuiApp) };
    let mut list = a.pages.lock().unwrap();
    if index >= 0 && (index as usize) < list.len() {
        list[index as usize].design_preview = t;
    }
}

/// Lua calls this to update a page's prompt after user edits
#[no_mangle]
pub extern "C" fn gui_set_page_prompt(app: *mut c_void, index: c_int, prompt: *const c_char) {
    if app.is_null() || prompt.is_null() { return; }
    let p = unsafe { CStr::from_ptr(prompt).to_string_lossy().to_string() };
    let a: &GuiApp = unsafe { &*(app as *mut GuiApp) };
    let mut list = a.pages.lock().unwrap();
    if index >= 0 && (index as usize) < list.len() {
        list[index as usize].prompt = p;
    }
}

#[no_mangle]
pub extern "C" fn gui_run(app_ptr: *mut c_void, _lua_state: *mut c_void) -> c_int {
    if app_ptr.is_null() { return -1; }
    let app_ptr = app_ptr as *mut GuiApp;

    gpui_platform::application().run(move |cx: &mut App| {
        cx.activate(true);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(Bounds::centered(None, size(px(1400.0), px(860.0)), cx))),
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
                    view: ViewState::Input,
                    edit_idx: 0,
                    edit_prompt: String::new(),
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
    view: ViewState,
    edit_idx: usize,
    edit_prompt: String,
}

impl DesignView {
    fn send(&mut self, cmd: &str, cx: &mut Context<Self>) {
        let a: &mut GuiApp = unsafe { &mut *self.app };
        if let Some(cb) = a.on_user_message {
            let s = CString::new("default").unwrap();
            let t = CString::new(cmd).unwrap();
            cb(s.as_ptr(), t.as_ptr(), a.user_data);
        }
        cx.notify();
    }

    fn start_split(&mut self, cx: &mut Context<Self>) {
        let req = self.input_text.trim().to_string();
        if req.is_empty() { return; }
        self.view = ViewState::PageList; // will be updated when pages arrive
        self.send(&format!("/split {}", req), cx);
    }

    fn edit_page(&mut self, idx: usize, cx: &mut Context<Self>) {
        let pages = unsafe { &*self.app }.pages.lock().unwrap().clone();
        if idx < pages.len() {
            self.edit_idx = idx;
            self.edit_prompt = pages[idx].prompt.clone();
            self.view = ViewState::SingleEditor;
            cx.notify();
        }
    }

    fn save_and_back(&mut self, cx: &mut Context<Self>) {
        // Save current prompt
        {
            let mut pages = unsafe { &*self.app }.pages.lock().unwrap();
            if self.edit_idx < pages.len() {
                pages[self.edit_idx].prompt = self.edit_prompt.clone();
            }
        }
        self.view = ViewState::PageList;
        cx.notify();
    }

    fn generate_page(&mut self, cx: &mut Context<Self>) {
        self.save_and_back(cx);
        let pages = unsafe { &*self.app }.pages.lock().unwrap().clone();
        if self.edit_idx < pages.len() {
            self.send(&format!("/generate_page {} {}", self.edit_idx, pages[self.edit_idx].name), cx);
        }
    }

    fn handle_key(&mut self, event: &KeyDownEvent, cx: &mut Context<Self>) {
        let k = event.keystroke.key_char.as_ref().unwrap_or(&event.keystroke.key);
        match k.as_str() {
            "backspace" | "delete" => { self.input_text.pop(); cx.notify(); }
            "enter" if self.view == ViewState::Input => self.start_split(cx),
            "space" => { self.input_text.push(' '); cx.notify(); }
            s if s.len() == 1 => { self.input_text.push_str(s); cx.notify(); }
            _ => {}
        }
    }
}

impl Render for DesignView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let bg = palette::bg_deep();
        let surface = palette::bg_surface();
        let _ = palette::bg_card();
        let border = palette::border();
        let text_pri = palette::text_primary();
        let text_sec = palette::text_secondary();
        let accent = palette::accent();

        let pages = unsafe { &*self.app }.pages.lock().unwrap().clone();

        // ── Left panel ──
        let left = v_flex().w(px(220.0)).h_full().bg(surface).border_r_1().border_color(border)
            .child(h_flex().px_4().py(px(16.0)).gap_2()
                .child(div().w(px(28.0)).h(px(28.0)).rounded_lg().bg(accent)
                    .flex().items_center().justify_center()
                    .child(div().child("M").text_color(gpui::Rgba { r: 1.0, g: 1.0, b: 1.0, a: 1.0 }).font_weight(FontWeight::BOLD).text_base().font_family(font_stack())))
                .child(div().child("my_design").text_color(text_pri).text_base().font_weight(FontWeight::SEMIBOLD).font_family(font_stack())))
            .child(v_flex().flex_1().px_3().py(px(12.0)).gap_1()
                .child(section_title("PROJECT"))
                .child(project_btn("📁 Current Project", true, cx))
                .child(div().h(px(6.0))))
            .child(div().px_4().py(px(12.0)).border_t_1().border_color(border)
                .child(div().child("v0.3 · Page Editor").text_color(text_sec).text_xs().font_family(font_stack())));

        // ── Main content ──
        let main = match &self.view {
            ViewState::Input => self.render_input(cx),
            ViewState::PageList => self.render_page_list(&pages, cx),
            ViewState::SingleEditor => self.render_editor(&pages, cx),
            ViewState::BatchGenerate => self.render_batch(&pages, cx),
        };

        h_flex().size_full().bg(bg).child(left).child(main)
    }
}

// ── Renderers ─────────────────────────────────────────────────

impl DesignView {
    fn render_input(&mut self, cx: &mut Context<Self>) -> Div {
        let text_pri = palette::text_primary();
        let text_sec = palette::text_secondary();
        let accent = palette::accent();
        let bg = palette::bg_deep();

        let display = if self.input_text.is_empty() {
            SharedString::from("e.g. Design a food delivery app...")
        } else { SharedString::from(self.input_text.as_str()) };
        let color = if self.input_text.is_empty() { text_sec } else { text_pri };
        let can = !self.input_text.trim().is_empty();

        v_flex().flex_1().h_full().bg(bg)
            .child(v_flex().flex_1().items_center().justify_center().gap_4()
                .child(div().w(px(80.0)).h(px(80.0)).rounded_2xl()
                    .bg(gpui::Rgba { r: 0.39, g: 0.38, b: 0.95, a: 0.15 })
                    .flex().items_center().justify_center()
                    .child(div().child("🚀").text_2xl()))
                .child(div().child("What are you building?").text_color(text_pri).text_xl().font_weight(FontWeight::SEMIBOLD).font_family(font_stack()))
                .child(div().child("Describe your project — AI will split it into pages.").text_color(text_sec).text_base().font_family(font_stack()))
                .child(
                    h_flex().gap_3().pt_4()
                        .child(div().w(px(400.0)).id("input")
                            .px_4().py(px(12.0)).rounded_xl()
                            .bg(palette::bg_card()).border_1().border_color(palette::border()).cursor_text()
                            .child(div().child(display.to_string()).text_color(color).text_base().font_family(font_stack()))
                            .on_key_down(cx.listener(|this, ev: &KeyDownEvent, _, cx| { this.handle_key(ev, cx); }))
                            .on_mouse_down(gpui::MouseButton::Left, |_, _, _| {}))
                        .child(div().px_5().py(px(12.0)).rounded_xl()
                            .bg(if can { accent } else { palette::bg_elevated() })
                            .hover(|s| if can { s.bg(palette::accent_hover()) } else { s }).cursor_pointer()
                            .on_mouse_down(gpui::MouseButton::Left, cx.listener(|this, _, _, cx| this.start_split(cx)))
                            .child(h_flex().gap_2().items_center()
                                .child(div().child("🔍").text_sm())
                                .child(div().child("Split").text_color(gpui::Rgba { r: 1.0, g: 1.0, b: 1.0, a: 1.0 }).text_sm().font_weight(FontWeight::SEMIBOLD).font_family(font_stack()))))
                ))
    }

    fn render_page_list(&mut self, pages: &[PageItem], cx: &mut Context<Self>) -> Div {
        let text_pri = palette::text_primary();
        let text_sec = palette::text_secondary();
        let accent = palette::accent();
        let card = palette::bg_card();
        let border = palette::border();
        let bg = palette::bg_deep();

        let mut items: Vec<AnyElement> = Vec::new();
        // Header
        items.push(v_flex().gap_1().px_6()
            .child(h_flex().gap_3().items_center()
                .child(div().child("📋").text_lg())
                .child(div().child("Pages").text_color(text_pri).text_lg().font_weight(FontWeight::SEMIBOLD).font_family(font_stack()))
                .child(div().child(format!("({})", pages.len())).text_color(text_sec).text_sm().font_family(font_stack())))
            .child(div().child("Click a page to edit its details, or generate all at once.").text_color(text_sec).text_sm().font_family(font_stack()))
            .into_any_element());

        // New project button
        items.push(h_flex().px_6().pt_2()
            .child(div().px_3().py(px(6.0)).rounded_md()
                .hover(|s| s.bg(card)).cursor_pointer()
                .on_mouse_down(gpui::MouseButton::Left, cx.listener(|this, _, _, cx| {
                    this.input_text.clear();
                    this.view = ViewState::Input;
                    unsafe { &*this.app }.pages.lock().unwrap().clear();
                    cx.notify();
                }))
                .child(h_flex().gap_2().items_center()
                    .child(div().child("＋").text_color(accent).text_sm())
                    .child(div().child("New Project").text_color(accent).text_sm().font_family(font_stack()))))
            .into_any_element());

        // Page list (with depth-based indentation for sub-flows)
        let mut prev_depth: usize = 0;
        for (i, p) in pages.iter().enumerate() {
            let icons = ["🛒", "📄", "🛵", "💳", "👤", "📦", "⚙️", "🏠", "📊", "🔐"];
            let icon = icons.get(i).unwrap_or(&"📄");
            let idx = i;
            let indent = p.depth * 24;
            // Show a branch connector if depth increases
            let branch = if p.depth > prev_depth {
                div().w(px(12.0)).child("└─").text_color(text_sec).text_xs().font_family(font_stack())
            } else { div().w(px(0.0)) };
            prev_depth = p.depth;

            items.push(
                h_flex()
                    .pl(px(indent as f32 + 6.0))  // indent based on depth
                    .pr(px(6.0)).py(px(10.0))
                    .mx(px(16.0)).rounded_lg()
                    .bg(if i % 2 == 0 { gpui::Rgba { r: 0.0, g: 0.0, b: 0.0, a: 0.0 } } else { card })
                    .hover(|s| s.bg(card)).cursor_pointer()
                    .on_mouse_down(gpui::MouseButton::Left, cx.listener(move |this, _, _, cx| this.edit_page(idx, cx)))
                    .gap_2().items_center()
                    .child(branch)
                    .child(div().w(px(18.0)).h(px(18.0)).rounded_md()
                        .bg(if p.selected { accent } else { gpui::Rgba { r: 0.0, g: 0.0, b: 0.0, a: 0.0 } })
                        .border_1().border_color(if p.selected { accent } else { border })
                        .flex().items_center().justify_center()
                        .child(if p.selected { div().child("✓").text_color(gpui::Rgba { r: 1.0, g: 1.0, b: 1.0, a: 1.0 }).text_xs() } else { div() })
                        .on_mouse_down(gpui::MouseButton::Left, cx.listener(move |this, _, _, cx| {
                            let mut list = unsafe { &*this.app }.pages.lock().unwrap();
                            if idx < list.len() { list[idx].selected = !list[idx].selected; }
                            cx.notify();
                        })))
                    .child(div().child(*icon).text_base())
                    .child(div().child(p.name.clone()).text_color(text_pri).text_sm().font_family(font_stack()).flex_1())
                    .child(if p.done { div().child("✅").text_sm() } else { div().child("").w(px(0.0)) })
                    .child(if p.depth > 0 { div().child("↳").text_color(text_sec).text_xs() } else { div() })
                    .child(div().px_3().py(px(4.0)).rounded_md().bg(palette::bg_elevated())
                        .child(div().child("Edit →").text_color(text_sec).text_xs().font_family(font_stack())))
                    .into_any_element()
            );
        }

        // Bottom: Skill + Generate
        let has_selected = pages.iter().any(|p| p.selected);
        items.push(divider_line(border).into_any_element());
        items.push(
            h_flex().px_6().gap_4().items_center()
                .child(h_flex().gap_2().items_center()
                    .child(div().child("Skill:").text_color(text_sec).text_xs().font_family(font_stack()))
                    .child(skill_pill("Shadcn/ui", true, accent)))
                .child(div().flex_1())
                .child(div().px_5().py(px(10.0)).rounded_xl()
                    .bg(if has_selected { accent } else { palette::bg_elevated() })
                    .hover(|s| if has_selected { s.bg(palette::accent_hover()) } else { s }).cursor_pointer()
                    .on_mouse_down(gpui::MouseButton::Left, cx.listener(|this, _, _, cx| {
                        if unsafe { &*this.app }.pages.lock().unwrap().iter().any(|p| p.selected) {
                            this.view = ViewState::BatchGenerate;
                            this.send("/generate_all", cx);
                        }
                    }))
                    .child(h_flex().gap_2().items_center()
                        .child(div().child("✨").text_sm())
                        .child(div().child(format!("Generate {} pages", pages.iter().filter(|p| p.selected).count()))
                            .text_color(gpui::Rgba { r: 1.0, g: 1.0, b: 1.0, a: 1.0 }).text_sm().font_weight(FontWeight::SEMIBOLD).font_family(font_stack()))))
                .into_any_element()
        );

        v_flex().flex_1().h_full().bg(bg).children(items)
    }

    fn render_editor(&mut self, pages: &[PageItem], cx: &mut Context<Self>) -> Div {
        let text_pri = palette::text_primary();
        let text_sec = palette::text_secondary();
        let accent = palette::accent();
        let card = palette::bg_card();
        let border = palette::border();
        let bg = palette::bg_deep();

        let page = pages.get(self.edit_idx);
        let name = page.map(|p| p.name.clone()).unwrap_or_else(|| String::from("Unknown"));
        let preview = page.map(|p| p.design_preview.clone()).unwrap_or_else(|| String::new());
        let has_preview = !preview.is_empty();
        let _can_gen = !self.edit_prompt.trim().is_empty();

        let display_prompt = if self.edit_prompt.is_empty() {
            SharedString::from("Describe what this page should include...")
        } else { SharedString::from(self.edit_prompt.as_str()) };
        let prompt_color = if self.edit_prompt.is_empty() { text_sec } else { text_pri };

        v_flex().flex_1().h_full().bg(bg)
            .child(v_flex().px_6().py(px(16.0)).gap_4()
                // Header with back button
                .child(h_flex().gap_3().items_center()
                    .child(div().px_3().py(px(6.0)).rounded_md()
                        .hover(|s| s.bg(card)).cursor_pointer()
                        .on_mouse_down(gpui::MouseButton::Left, cx.listener(|this, _, _, cx| { this.save_and_back(cx); }))
                        .child(h_flex().gap_2().items_center()
                            .child(div().child("←").text_color(text_sec).text_sm())
                            .child(div().child("Back to Pages").text_color(text_sec).text_sm().font_family(font_stack()))))
                    .child(div().child("📄").text_lg())
                    .child(div().child(name).text_color(text_pri).text_lg().font_weight(FontWeight::SEMIBOLD).font_family(font_stack()))
                    .child(if has_preview { div().child("✅ Saved").text_color(palette::green()).text_sm().font_family(font_stack()) } else { div() }))
                // Prompt input
                .child(
                    v_flex().gap_2()
                        .child(div().child("Page Requirements").text_color(text_sec).text_xs().font_weight(FontWeight::BOLD).font_family(font_stack()))
                        .child(div().id("editor-input")
                            .px_4().py(px(12.0)).rounded_lg().h(px(100.0))
                            .bg(card).border_1().border_color(border).cursor_text()
                            .child(div().child(display_prompt.to_string()).text_color(prompt_color).text_sm().font_family(font_stack()))
                            .on_key_down(cx.listener(|this, ev: &KeyDownEvent, _, cx| {
                                let k = ev.keystroke.key_char.as_ref().unwrap_or(&ev.keystroke.key);
                                match k.as_str() {
                                    "backspace" | "delete" => { this.edit_prompt.pop(); cx.notify(); }
                                    "space" => { this.edit_prompt.push(' '); cx.notify(); }
                                    s if s.len() == 1 => { this.edit_prompt.push_str(s); cx.notify(); }
                                    _ => {}
                                }
                            }))
                            .on_mouse_down(gpui::MouseButton::Left, |_, _, _| {}))
                )
                // Action buttons
                .child(
                    h_flex().gap_3()
                        .child(div().px_4().py(px(8.0)).rounded_lg().bg(accent)
                            .hover(|s| s.bg(palette::accent_hover())).cursor_pointer()
                            .on_mouse_down(gpui::MouseButton::Left, cx.listener(|this, _, _, cx| {
                                this.save_and_back(cx);
                                this.generate_page(cx);
                            }))
                            .child(h_flex().gap_2().items_center()
                                .child(div().child("✨").text_sm())
                                .child(div().child("Generate This Page")
                                    .text_color(gpui::Rgba { r: 1.0, g: 1.0, b: 1.0, a: 1.0 }).text_sm().font_weight(FontWeight::SEMIBOLD).font_family(font_stack()))))
                        .child(div().px_4().py(px(8.0)).rounded_lg().bg(card).border_1().border_color(border)
                            .hover(|s| s.bg(palette::bg_elevated())).cursor_pointer()
                            .on_mouse_down(gpui::MouseButton::Left, cx.listener(|this, _, _, cx| { this.save_and_back(cx); }))
                            .child(div().child("💾 Save & Back").text_color(text_pri).text_sm().font_family(font_stack())))
                )
                // Design preview
                .child(if has_preview {
                    v_flex().gap_2()
                        .child(divider_line(border))
                        .child(div().child("Design Preview").text_color(text_sec).text_xs().font_weight(FontWeight::BOLD).font_family(font_stack()))
                        .child(div().px_4().py(px(16.0)).rounded_lg()
                            .bg(gpui::Rgba { r: 0.39, g: 0.38, b: 0.95, a: 0.06 })
                            .border_1().border_color(palette::accent_dim())
                            .child(div().child(preview).text_color(text_pri).text_sm().font_family(font_stack())))
                        .into_any_element()
                } else { div().into_any_element() })
            )
    }

    fn render_batch(&mut self, pages: &[PageItem], _cx: &mut Context<Self>) -> Div {
        let text_pri = palette::text_primary();
        let text_sec = palette::text_secondary();
        let bg = palette::bg_deep();
        let card = palette::bg_card();
        let icons = ["🛒", "📄", "🛵", "💳", "👤", "📦", "⚙️", "🏠", "📊", "🔐"];
        let done_count = pages.iter().filter(|p| p.done).count();

        v_flex().flex_1().h_full().bg(bg)
            .child(v_flex().px_6().py(px(20.0)).gap_4()
                .child(v_flex().gap_1()
                    .child(h_flex().gap_3().items_center()
                        .child(div().child("🎨").text_lg())
                        .child(div().child("Generating Designs").text_color(text_pri).text_lg().font_weight(FontWeight::SEMIBOLD).font_family(font_stack())))
                    .child(div().child(format!("{}/{} pages completed", done_count, pages.len())).text_color(text_sec).text_sm().font_family(font_stack())))
                .children(pages.iter().enumerate().map(|(i, p)| {
                    let icon = icons.get(i).unwrap_or(&"📄");
                    let status = if p.done { "✅" } else { "⏳" };
                    let bg_c = if p.done { gpui::Rgba { r: 0.13, g: 0.77, b: 0.37, a: 0.08 } } else { card };
                    h_flex().px_4().py(px(10.0)).rounded_lg().bg(bg_c).gap_3().items_center()
                        .child(div().child(status).text_sm())
                        .child(div().child(*icon).text_base())
                        .child(div().child(p.name.clone()).text_color(text_pri).text_sm().font_family(font_stack()).flex_1())
                        .child(if p.done { div().child("Done").text_color(palette::green()).text_xs().font_family(font_stack()) } else { div().child("Generating...").text_color(text_sec).text_xs().font_family(font_stack()) })
                        .into_any_element()
                })))
    }
}

// ── Helpers ───────────────────────────────────────────────────

fn section_title(text: &str) -> impl IntoElement {
    div().child(text.to_uppercase()).text_xs().font_weight(FontWeight::BOLD)
        .text_color(palette::text_secondary()).font_family(font_stack())
}

fn project_btn(text: &str, active: bool, _cx: &mut Context<'_, DesignView>) -> impl IntoElement {
    let t = SharedString::from(text);
    let bg = if active { palette::accent_dim() } else { gpui::Rgba { r: 0.0, g: 0.0, b: 0.0, a: 0.0 } };
    div().px_3().py(px(8.0)).rounded_md().bg(bg).cursor_pointer()
        .child(div().child(t).text_color(palette::text_primary()).text_sm().font_family(font_stack()))
}

fn skill_pill(text: &str, active: bool, accent: Rgba) -> impl IntoElement {
    let t = SharedString::from(text);
    let bg = if active { accent } else { palette::bg_elevated() };
    let tc = if active { gpui::Rgba { r: 1.0, g: 1.0, b: 1.0, a: 1.0 } } else { palette::text_secondary() };
    div().px_3().py(px(4.0)).rounded_full().bg(bg)
        .child(div().child(t).text_color(tc).text_xs().font_weight(FontWeight::MEDIUM).font_family(font_stack()))
}

fn divider_line(border: Rgba) -> impl IntoElement {
    div().h(px(1.0)).mx(px(16.0)).bg(border)
}

#[no_mangle]
pub extern "C" fn gui_refresh(_app: *mut c_void) {}
