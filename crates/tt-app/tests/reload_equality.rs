//! The spec's drift guard (OQ-9 "Accepted risk"): random sessions through the write path;
//! after a reload from disk the serialised State equals the in-memory State.

mod common;

use common::Scratch;
use proptest::prelude::*;
use tt_app::{App, AppConfig};
use tt_core::testing::{advance, arb_start_ctx, arb_steps, materialize};

proptest! {
    #![proptest_config(ProptestConfig { cases: 32, ..ProptestConfig::default() })]

    #[test]
    fn reloaded_state_equals_the_in_memory_state(start in arb_start_ctx(), steps in arb_steps(150)) {
        let scratch = Scratch::new();
        let mut ctx = start;
        let app = App::open(AppConfig::new(scratch.db()), &ctx).unwrap();
        for step in &steps {
            ctx = advance(&ctx, step.advance_secs);
            let cmd = materialize(&step.op, &app.snapshot().unwrap(), &ctx);
            let _ = app.execute(&ctx, cmd); // rejections are part of the session
        }
        let in_memory = app.snapshot().unwrap();
        drop(app);

        let reopened = App::open(AppConfig::new(scratch.db()), &ctx).unwrap();
        prop_assert_eq!(reopened.status().broken, None);
        let reloaded = reopened.snapshot().unwrap();
        prop_assert_eq!(
            serde_json::to_string(&reloaded).unwrap(),
            serde_json::to_string(&in_memory).unwrap()
        );
    }
}
