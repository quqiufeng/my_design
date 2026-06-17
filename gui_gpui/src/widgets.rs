// my_design — 基础布局辅助

use gpui::*;

/// 水平 flex 容器
pub fn h_flex() -> Div {
    div().flex().flex_row()
}

/// 垂直 flex 容器
pub fn v_flex() -> Div {
    div().flex().flex_col()
}
