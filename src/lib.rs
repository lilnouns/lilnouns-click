use url::Url;
use worker::{event, Context, Env, Request, Response, Result, Router};

mod handlers;
mod helpers;
mod queries;

#[event(fetch)]
async fn main(req: Request, env: Env, _ctx: Context) -> Result<Response> {
  let router = Router::new();

  router
    .get("/", |_, _| {
      Response::redirect(Url::parse(
        "https://lilnouns.camp?utm_source=farcaster&utm_medium=social",
      )?)
    })
    .get_async("/:sqid", handlers::handle_redirect)
    .get_async("/:sqid/og.png", handlers::generate_og_image)
    .post_async("/", handlers::handle_creation)
    .run(req, env)
    .await
}
