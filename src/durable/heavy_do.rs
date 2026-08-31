use tower_service::Service;
use worker::{durable_object, DurableObject, Env, HttpRequest, Request, Response, Result, State};

use crate::build_http_app;

/// Durable Object used to run CPU-heavy API flows with a higher CPU budget.
///
/// This DO intentionally reuses the existing axum router so we don't have to duplicate business
/// logic in a separate code path.
#[durable_object]
pub struct HeavyDo {
    state: State,
    env: Env,
}

impl DurableObject for HeavyDo {
    fn new(state: State, env: Env) -> Self {
        Self { state, env }
    }

    async fn fetch(&self, req: Request) -> Result<Response> {
        console_error_panic_hook::set_once();
        let _ = console_log::init_with_level(crate::handlers::log_level(&self.env));

        let _ = &self.state;

        let http_req: HttpRequest = req.try_into()?;
        let mut app = build_http_app(self.env.clone(), &http_req);
        let http_resp = app.call(http_req).await?;
        http_resp.try_into()
    }
}
