use axum::{
  response::{IntoResponse, Redirect},
  routing::get,
  Router as AxumRouter,
};
use axum_cloudflare_adapter::{to_axum_request, to_worker_response, EnvWrapper};
use routes::handle_creation;
use tower_service::Service;
use worker::{event, Context, Env, Request, Response, Result};

use crate::routes::handle_redirect;

mod queries;
mod routes;
mod utils;

#[derive(Clone)]
struct AxumState {
  env_wrapper: EnvWrapper,
}

#[event(fetch)]
async fn fetch(req: Request, env: Env, _ctx: Context) -> Result<Response> {
  // Wrap the environment
  let state = AxumState {
    env_wrapper: EnvWrapper::new(env),
  };

  // Create the Axum router
  let mut router: AxumRouter = AxumRouter::new()
    .route(
      "/",
      get(|| async {
        Redirect::temporary("https://lilnouns.camp?utm_source=farcaster&utm_medium=social")
      })
      .post(handle_creation),
    )
    .route("/:sqid", get(handle_redirect))
    .route("/:sqid/og.png", get(routes::handle_og_image))
    // .route("/app/:sqid", get(routes::handle_mini_app))
    .with_state(state);

  // Convert the Cloudflare Request to an Axum Request
  let axum_request = to_axum_request(req).await.unwrap();

  // Handle the request using Axum
  let axum_response = router.call(axum_request).await.unwrap();

  // Convert the Axum Response to a Cloudflare Response
  let response = to_worker_response(axum_response).await.unwrap();
  Ok(response)
}
