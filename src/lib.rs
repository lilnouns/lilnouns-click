use serde::{Deserialize, Serialize};
use sqids::Sqids;
use url::Url;
use worker::{event, Context, Env, Request, Response, ResponseBody, Result, RouteContext, Router};

use crate::{
  Community::LilNouns,
  Platform::{Ethereum, MetaGov, PropLot},
};

#[derive(Debug, Serialize, Deserialize)]
struct UrlPayload {
  pub url: String,
  pub sqid: Option<String>,
}

#[derive(Debug, PartialEq)]
enum Community {
  LilNouns = 1,
}

#[derive(Debug, PartialEq)]
enum Platform {
  Ethereum = 1,
  PropLot = 2,
  MetaGov = 3,
}

#[event(fetch)]
async fn main(req: Request, env: Env, _ctx: Context) -> Result<Response> {
  let router = Router::new();

  router
    .get("/", |_, _| Response::ok("Hello, World!"))
    .get_async("/:sqid", handle_redirect)
    .post_async("/", handle_creation)
    .run(req, env)
    .await
}

async fn handle_redirect<D>(_req: Request, ctx: RouteContext<D>) -> Result<Response> {
  if let Some(sqid) = ctx.param("sqid") {
    let sqids = Sqids::default();
    let numbers = sqids.decode(&sqid);

    let community = match numbers[0] {
      1 => Some(LilNouns),
      _ => None,
    };

    let platform = match numbers[1] {
      1 => Some(Ethereum),
      2 => Some(PropLot),
      3 => Some(MetaGov),
      _ => None,
    };

    let url = match (community, platform) {
      (Some(LilNouns), Some(Ethereum)) => {
        format!("{}/{}", "https://lilnouns.wtf/vote", numbers[2])
      }
      (Some(LilNouns), Some(PropLot)) => {
        format!("{}/{}", "https://lilnouns.proplot.wtf/idea", numbers[2])
      }
      (Some(LilNouns), Some(MetaGov)) => {
        format!("{}/{}", "https://lilnouns.wtf/vote/nounsdao", numbers[2])
      }
      _ => String::new(),
    };

    let html_doc = format!(
      "
        <html>
            <head>
                <title>Your Site Title</title>
                \
       <meta property=\"og:title\" content=\"Your Site Title\" />
                <meta property=\"og:description\" \
       content=\"A description of your site.\" />
                <meta property=\"og:image\" content=\"https://your-site.com/image.png\" \
       />
                <meta property=\"og:url\" content=\"{}\" />
                <meta http-equiv=\"refresh\" content=\"5; url={}\" />
            </head>
            <body>
                <h1>Redirecting...</h1>
            </body>
        </html>",
      url, url
    );

    return Response::from_body(ResponseBody::Body(html_doc.as_bytes().to_vec()));
  }

  Response::error("Bad Request", 400)
}

async fn handle_creation<D>(mut req: Request, _ctx: RouteContext<D>) -> Result<Response> {
  let sqids = Sqids::default();
  let mut numbers: Vec<u64> = Vec::new();

  if let Ok(payload) = req.json::<UrlPayload>().await {
    let url = Url::parse(&*payload.url).expect("Invalid URL");

    return match url.host_str() {
      Some("lilnouns.wtf") | Some("www.lilnouns.wtf") => {
        let segments: Vec<_> = url
          .path_segments()
          .expect("Cannot get path segments")
          .filter(|segment| !segment.is_empty())
          .collect();

        if segments.is_empty() || segments[0] != "vote" {
          return Response::error("Bad Request", 400);
        }

        if segments[1] == "nounsdao" {
          numbers.push(LilNouns as u64);
          numbers.push(MetaGov as u64);
          numbers.push(segments[2].parse::<u32>().unwrap().try_into().unwrap());
        } else {
          numbers.push(LilNouns as u64);
          numbers.push(Ethereum as u64);
          numbers.push(segments[1].parse::<u32>().unwrap().try_into().unwrap());
        }

        Response::from_json(&UrlPayload {
          url: url.into(),
          sqid: Some(sqids.encode(&*numbers).unwrap()),
        })
      }
      Some("lilnouns.proplot.wtf") | Some("www.lilnouns.proplot.wtf") => {
        numbers.push(LilNouns as u64);

        let segments: Vec<_> = url
          .path_segments()
          .expect("Cannot get path segments")
          .filter(|segment| !segment.is_empty())
          .collect();

        if segments[0] == "idea" {
          numbers.push(LilNouns as u64);
          numbers.push(PropLot as u64);
        } else {
          return Response::error("Bad Request", 400);
        }

        Response::from_json(&UrlPayload {
          url: url.into(),
          sqid: Some(sqids.encode(&*numbers).unwrap()),
        })
      }
      _ => Response::error("Bad Request", 400),
    };
  }

  Response::error("Bad Request", 400)
}
