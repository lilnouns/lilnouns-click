use url::Url;
use worker::{event, Context, Env, Request, Response, Result, Router};

mod queries;
mod routes;
mod utils;

#[event(fetch)]
async fn main(req: Request, env: Env, _ctx: Context) -> Result<Response> {
  let router = Router::new();

  router
    .get("/", |_, _| {
      Response::redirect(Url::parse(
        "https://lilnouns.camp?utm_source=farcaster&utm_medium=social",
      )?)
    })
    .get_async("/:sqid", routes::handle_redirect)
    .get_async("/:sqid/og.png", routes::handle_og_image)
    .post_async("/", routes::handle_creation)
    .on_async("/app/:sqid", routes::handle_mini_app)
    .run(req, env)
    .await
}
