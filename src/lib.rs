use url::Url;
use worker::{event, Context, Env, Request, Response, Result, Router};

mod handlers;
mod helpers;
mod queries;

#[event(fetch)]
async fn main(req: Request, env: Env, _ctx: Context) -> Result<Response> {
  Router::new()
    .get("/", |_, _| {
      Response::redirect(Url::parse(
        "https://lilnouns.camp?utm_source=farcaster&utm_medium=social",
      )?)
    })
    .get_async("/:sqid", handlers::generate_redirect_page)
    .get_async("/:sqid/og.png", handlers::generate_og_image_url)
    .post_async("/", handlers::generate_from_url)
    .get_async(
      "/.well-known/farcaster.json",
      handlers::generate_farcaster_manifest,
    )
    .run(req, env)
    .await
}
