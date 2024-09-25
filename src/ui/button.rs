// src/ui/button.rs

use druid::{widget::Controller, Env, Event, EventCtx, UpdateCtx, Widget};

use crate::models::Tweak;

pub struct ButtonController;

impl<W: Widget<Tweak>> Controller<Tweak, W> for ButtonController {
    fn update(
        &mut self,
        child: &mut W,
        ctx: &mut UpdateCtx,
        old_data: &Tweak,
        data: &Tweak,
        env: &Env,
    ) {
        if old_data.applying != data.applying {
            ctx.request_paint();
        }
        child.update(ctx, old_data, data, env);
    }

    fn event(
        &mut self,
        child: &mut W,
        ctx: &mut EventCtx,
        event: &Event,
        data: &mut Tweak,
        env: &Env,
    ) {
        // Disable the button if applying
        if data.applying {
            return;
        }
        child.event(ctx, event, data, env);
    }
}
