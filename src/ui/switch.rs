// src/ui/button.rs

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
            ctx.request_paint();

            let sink = ctx.get_external_handle();
            let tweak_id = data.id;
            let enabled = data.enabled;
            let data_clone = data.clone();

            std::thread::spawn(move || {
                let success = if !enabled {
                    match data_clone.apply() {
                        Ok(_) => true,
                        Err(e) => {
                            eprintln!("Failed to apply tweak '{}': {}", data_clone.name, e);
                            false
                        }
                    }
                } else {
                    match data_clone.revert() {
                        Ok(_) => false,
                        Err(e) => {
                            eprintln!("Failed to revert tweak '{}': {}", data_clone.name, e);
                            true
                        }
                    }
                };

                sink.submit_command(SET_APPLYING, (tweak_id, false), Target::Auto)
                    .expect("Failed to submit command");

                if success {
                    // Update data.enabled
                    sink.submit_command(UPDATE_TWEAK_ENABLED, (tweak_id, !enabled), Target::Auto)
                        .expect("Failed to submit command");
                }
            });
        }
        self.child.event(ctx, event, &mut data.enabled, env);

        if let Event::MouseDown(_) = event {
            if data.applying {
                // Do nothing if already applying
                return;
            }

            data.applying = true;
            ctx.request_paint();

            let sink = ctx.get_external_handle();
            let tweak_id = data.id;
            let enabled = data.enabled;
            let data_clone = data.clone();

            std::thread::spawn(move || {
                let result = if !enabled {
                    data_clone.apply()
                } else {
                    data_clone.revert()
                };

                let success = result.is_ok();

                if let Err(ref e) = result {
                    println!("Failed to apply/revert tweak '{}': {}", data_clone.name, e);
                } else {
                    println!("Applied/Reverted tweak '{}'", data_clone.name);
                }

                sink.submit_command(SET_APPLYING, (tweak_id, false), Target::Auto)
                    .expect("Failed to submit command");

                if success {
                    // Update data.enabled
                    sink.submit_command(UPDATE_TWEAK_ENABLED, (tweak_id, !enabled), Target::Auto)
                        .expect("Failed to submit command");
                }
            });
        }
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
