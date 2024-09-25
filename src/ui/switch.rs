// src/ui/switch.rs

use std::thread;

use druid::{
    widget::Switch, BoxConstraints, Env, Event, EventCtx, LayoutCtx, LifeCycle, LifeCycleCtx,
    PaintCtx, Size, Target, UpdateCtx, Widget,
};

use crate::{
    actions::TweakAction,
    constants::{SET_APPLYING, UPDATE_TWEAK_ENABLED},
    models::Tweak,
};

pub struct TweakSwitch {
    pub child: Switch,
}

impl Widget<Tweak> for TweakSwitch {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut Tweak, env: &Env) {
        if let Event::MouseDown(_) = event {
            if data.applying {
                // Do nothing if already applying
                return;
            }

            data.applying = true;
            tracing::debug!("Tweak '{}' is now applying.", data.name);
            ctx.request_paint();

            let sink = ctx.get_external_handle();
            let tweak_id = data.id;
            let enabled = data.enabled;
            let mut data_clone = data.clone();

            thread::spawn(move || {
                let success = if !enabled {
                    match data_clone.apply() {
                        Ok(_) => true,
                        Err(e) => {
                            tracing::error!("Failed to apply tweak '{}': {}", data_clone.name, e);
                            false
                        }
                    }
                } else {
                    match data_clone.revert() {
                        Ok(_) => false,
                        Err(e) => {
                            tracing::error!("Failed to revert tweak '{}': {}", data_clone.name, e);
                            true
                        }
                    }
                };

                // Always set 'applying' to false
                sink.submit_command(SET_APPLYING, (tweak_id, false), Target::Auto)
                    .expect("Failed to submit SET_APPLYING command");

                if success {
                    let new_enabled = !enabled;
                    sink.submit_command(
                        UPDATE_TWEAK_ENABLED,
                        (tweak_id, new_enabled),
                        Target::Auto,
                    )
                    .expect("Failed to submit UPDATE_TWEAK_ENABLED command");
                    tracing::debug!(
                        "Tweak '{}' successfully toggled to {}.",
                        data_clone.name,
                        new_enabled
                    );
                } else {
                    // Optionally, notify user of failure
                    tracing::error!(
                        "Tweak '{}' failed to toggle. Keeping enabled as {}.",
                        data_clone.name,
                        enabled
                    );
                }
            });
        }
        self.child.event(ctx, event, &mut data.enabled, env);
    }

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &Tweak, env: &Env) {
        self.child.lifecycle(ctx, event, &data.enabled, env);
    }

    fn update(&mut self, ctx: &mut UpdateCtx, old_data: &Tweak, data: &Tweak, env: &Env) {
        self.child
            .update(ctx, &old_data.enabled, &data.enabled, env);
    }

    fn layout(
        &mut self,
        ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        data: &Tweak,
        env: &Env,
    ) -> Size {
        self.child.layout(ctx, bc, &data.enabled, env)
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &Tweak, env: &Env) {
        self.child.paint(ctx, &data.enabled, env);
    }
}
