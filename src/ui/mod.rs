// src/ui/mod.rs

mod button;
mod switch;
pub mod widgets;

use druid::{
    widget::{Flex, Label, Scroll},
    Widget, WidgetExt,
};
use widgets::make_tweak_widget;

use crate::models::AppState;

pub fn build_root_widget() -> impl Widget<AppState> {
    let list = druid::widget::List::new(make_tweak_widget);
    let scroll = Scroll::new(list)
        .vertical()
        .padding(10.0)
        .expand_height()
        .lens(AppState::tweak_list);

    let info_bar = Label::new(|data: &AppState, _: &_| {
        let count = data
            .tweak_list
            .iter()
            .filter(|tweak| tweak.enabled && tweak.requires_restart)
            .count();
        if count > 0 {
            format!("{} tweaks pending restart", count)
        } else {
            "".to_string()
        }
    })
    .padding(5.0);

    Flex::column()
        .with_flex_child(scroll, 1.0)
        .with_child(info_bar)
}
