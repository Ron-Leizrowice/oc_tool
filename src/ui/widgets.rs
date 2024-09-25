// src/ui/widgets.rs

use druid::{
    widget::{Button, Controller, CrossAxisAlignment, Either, Flex, Label, Switch},
    Data, Env, Event, EventCtx, Target, Widget, WidgetExt,
};

use super::{button::ButtonController, switch::TweakSwitch};
use crate::{
    actions::TweakAction,
    constants::{SET_APPLYING, UPDATE_TWEAK_ENABLED},
    models::Tweak,
};

#[derive(Clone, Data, PartialEq)]
pub enum WidgetType {
    Switch,
    Button,
}

pub fn make_tweak_widget() -> impl Widget<Tweak> {
    // Common label for all tweaks
    let label = Label::new(|data: &Tweak, _: &_| data.name.clone())
        .fix_width(250.0)
        .padding(5.0);

    // Placeholder for the control widget (Switch or Button)
    let control = Either::new(
        |data: &Tweak, _: &_| data.widget == WidgetType::Switch,
        make_switch(),
        make_command_button(),
    );

    let applying_label = Label::new(|data: &Tweak, _: &_| {
        if data.applying {
            "applying".to_string()
        } else {
            "".to_string()
        }
    })
    .fix_width(70.0) // Set fixed width to prevent layout shift
    .padding(5.0);

    Flex::row()
        .with_child(label)
        .with_flex_spacer(1.0)
        .with_child(control)
        .with_child(applying_label)
        .cross_axis_alignment(CrossAxisAlignment::Center)
        .controller(TweakController::new())
}

fn make_switch() -> impl Widget<Tweak> {
    TweakSwitch {
        child: Switch::new(),
    }
}

fn make_command_button() -> impl Widget<Tweak> {
    Button::new("Apply")
        .on_click(|ctx, data: &mut Tweak, _env| {
            if data.applying {
                return;
            }
            data.applying = true;
            ctx.request_paint();

            let sink = ctx.get_external_handle();
            let tweak_id = data.id;
            let data_clone = data.clone();

            std::thread::spawn(move || {
                let result = data_clone.apply();

                if let Err(ref e) = result {
                    println!("Failed to execute command '{}': {}", data_clone.name, e);
                } else {
                    println!("Executed command '{}'", data_clone.name);
                }

                sink.submit_command(SET_APPLYING, (tweak_id, false), Target::Auto)
                    .expect("Failed to submit command");
            });
        })
        .controller(ButtonController)
}

// Controller to handle apply and revert actions
pub struct TweakController;

impl TweakController {
    pub fn new() -> Self {
        Self
    }
}

impl<W: Widget<Tweak>> Controller<Tweak, W> for TweakController {
    fn event(
        &mut self,
        child: &mut W,
        ctx: &mut EventCtx,
        event: &Event,
        data: &mut Tweak,
        env: &Env,
    ) {
        if let Event::Command(cmd) = event {
            if let Some((tweak_id, applying)) = cmd.get(SET_APPLYING) {
                if *tweak_id == data.id {
                    data.applying = *applying;
                    ctx.request_paint();
                }
            } else if let Some((tweak_id, enabled)) = cmd.get(UPDATE_TWEAK_ENABLED) {
                if *tweak_id == data.id {
                    data.enabled = *enabled;
                    ctx.request_paint();
                }
            }
        }
        child.event(ctx, event, data, env);
    }
}
