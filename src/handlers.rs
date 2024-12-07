use html_escape::encode_safe;
use html_minifier::minify;
use serde::{Deserialize, Serialize};
use sqids::Sqids;
use url::Url;
use worker::{Request, Response, ResponseBody, RouteContext};

use crate::{
  handlers::{
    Community::LilNouns,
    Platform::{Ethereum, LilCamp, MetaGov, PropLot},
  },
  helpers::create_og_image,
  queries::{fetch_lil_nouns_data, fetch_meta_gov_data, fetch_prop_lot_data},
};

#[derive(Debug, Serialize, Deserialize)]
struct UrlPayload {
  pub url: String,
  pub sqid: Option<String>,
}

#[derive(Debug, PartialEq)]
pub enum Community {
  LilNouns = 1,
}

#[derive(Debug, PartialEq)]
pub enum Platform {
  Ethereum = 1,
  PropLot = 2,
  MetaGov = 3,
  LilCamp = 4,
}

struct OpenGraphMeta {
  title: String,
  description: String,
  image: String,
  url: String,
}

impl OpenGraphMeta {
  fn to_html(&self) -> String {
    let farcaster_meta = format!(
      r#"
      <meta property="fc:frame" content="vNext" />
      <meta property="fc:frame:image" content="{image}\" />
      "#,
      image = self.image,
    );

    format!(
      r#"
      <meta property="og:site_name" content="Lil Nouns">
      <meta property="og:url" content="{url}">
      <meta property="og:type" content="website">
      <meta property="og:title" content="{title}">
      <meta property="og:description" content="{description}">
      <meta property="og:image" content="{image}">
      <meta name="twitter:card" content="summary_large_image">
      <meta name="twitter:title" content="{title}">
      <meta name="twitter:description" content="{description}">
      <meta name="twitter:image" content="{image}">
      {farcaster_meta}
      "#,
      url = self.url,
      title = encode_safe(&self.title),
      description = encode_safe(&self.description),
      image = self.image,
      farcaster_meta = farcaster_meta,
    )
  }
}

pub async fn handle_redirect<D>(req: Request, ctx: RouteContext<D>) -> worker::Result<Response> {
  if let Some(sqid) = ctx.param("sqid") {
    let ga_id = ctx.secret("GA_ID")?;
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
      4 => Some(LilCamp),
      _ => None,
    };

    let (url, title, description, image) = match (community, platform) {
      (Some(LilNouns), Some(Ethereum)) => {
        let url = format!(
          "{}/{}?utm_source=farcaster&utm_medium=social&utm_campaign=governor&\
           utm_content=proposal_{}",
          "https://lilnouns.wtf/vote", numbers[2], numbers[2]
        );
        let (title, description) = fetch_lil_nouns_data(&ctx.env, numbers[2]).await?;
        let image = req
          .url()?
          .join(format!("{}/og.png", sqid).as_str())?
          .to_string();
        (url, title, description, image)
      }
      (Some(LilNouns), Some(PropLot)) => {
        let url = format!(
          "{}/{}?utm_source=farcaster&utm_medium=social&utm_campaign=proplot&utm_content=idea_{}",
          "https://lilnouns.proplot.wtf/idea", numbers[2], numbers[2]
        );
        let (title, description) = fetch_prop_lot_data(&ctx.env, numbers[2]).await?;
        let image = req
          .url()?
          .join(format!("{}/og.png", sqid).as_str())?
          .to_string();
        (url, title, description, image)
      }
      (Some(LilNouns), Some(MetaGov)) => {
        let url = format!(
          "{}/{}?utm_source=farcaster&utm_medium=social&utm_campaign=metagov&\
           utm_content=proposal_{}",
          "https://lilnouns.wtf/vote/nounsdao", numbers[2], numbers[2]
        );
        let (title, description) = fetch_meta_gov_data(&ctx.env, numbers[2]).await?;
        let image = req
          .url()?
          .join(format!("{}/og.png", sqid).as_str())?
          .to_string();
        (url, title, description, image)
      }
      (Some(LilNouns), Some(LilCamp)) => {
        let url = format!(
          "{}/{}?utm_source=farcaster&utm_medium=social&utm_campaign=governor&\
           utm_content=proposal_{}",
          "https://lilnouns.camp/proposals", numbers[2], numbers[2]
        );
        let (title, description) = fetch_lil_nouns_data(&ctx.env, numbers[2]).await?;
        let image = req
          .url()?
          .join(format!("{}/og.png", sqid).as_str())?
          .to_string();
        (url, title, description, image)
      }
      _ => (String::new(), String::new(), String::new(), String::new()),
    };

    let og_meta = OpenGraphMeta {
      title: title.clone(),
      description: description.clone(),
      image,
      url: url.clone(),
    };

    let html_doc = format!(
      r#"
        <!DOCTYPE html>
        <html lang="en">
        <head>
            <meta charset="UTF-8">
            <meta name="viewport" content="width=device-width, initial-scale=1.0">
            {og_meta}
            <meta http-equiv="refresh" content="3; url={url}" />

            <title>{title}</title>
            <meta name="description" content="{description}">

            <link rel="preconnect" href="https://fonts.googleapis.com">
            <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
            <link href="https://fonts.googleapis.com/css2?family=Londrina+Solid:wght@100;300;400;900&display=swap" rel="stylesheet">

            <!-- Google tag (gtag.js) -->
            <script async src="https://www.googletagmanager.com/gtag/js?id={ga_id}"></script>
            <script>
              window.dataLayer = window.dataLayer || [];
              function gtag(){{dataLayer.push(arguments);}}
              gtag('js', new Date());

              gtag('config', '{ga_id}');
            </script>
        </head>
        <body style="margin: 0; padding: 0; display: flex; justify-content: center; align-items: center; height: 100vh; background-color: #f0f0f0; font-family: 'Londrina Solid', cursive;">
            <div style="text-align: center;">
                <div style="padding: 20px;">
                    <img src="https://lilnouns.wtf/static/media/lil-loading-skull.b7a846e1.gif" alt="Loading Skull" style="width: 192px; height: 192px;">
                    <p style="margin-top: 10px; font-size: 24px; font-weight: bold;">Redirecting...</p>
                    <p><a style="font-size: 24px; text-decoration: none;" href="{url}">{title}</a></p>
                </div>
            </div>
        </body>
        </html>
    "#,
      og_meta = og_meta.to_html(),
      url = url,                               // Page URL
      title = encode_safe(&title),             // Page Title
      description = encode_safe(&description), // Page Description
      ga_id = ga_id,                           // Google Analytics ID
    );

    let minified_html = minify(html_doc).expect("Failed to minify HTML");

    let response = Response::from_body(ResponseBody::Body(minified_html.as_bytes().to_vec()));

    return match response {
      Ok(mut res) => {
        res.headers_mut().set("Content-Type", "text/html")?;
        return Ok(res);
      }
      Err(e) => Err(e),
    };
  }

  Response::error("Bad Request", 400)
}

pub async fn handle_og_image<D>(_req: Request, ctx: RouteContext<D>) -> worker::Result<Response> {
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
      4 => Some(LilCamp),
      _ => None,
    };

    let image = match (community, platform) {
      (Some(LilNouns), Some(Ethereum)) => {
        let (title, description) = fetch_lil_nouns_data(&ctx.env, numbers[2]).await?;
        create_og_image(numbers[2], &title.to_uppercase(), &description, Ethereum)
      }
      (Some(LilNouns), Some(PropLot)) => {
        let (title, description) = fetch_prop_lot_data(&ctx.env, numbers[2]).await?;
        create_og_image(numbers[2], &title.to_uppercase(), &description, PropLot)
      }
      (Some(LilNouns), Some(MetaGov)) => {
        let (title, description) = fetch_meta_gov_data(&ctx.env, numbers[2]).await?;
        create_og_image(numbers[2], &title.to_uppercase(), &description, MetaGov)
      }
      (Some(LilNouns), Some(LilCamp)) => {
        let (title, description) = fetch_lil_nouns_data(&ctx.env, numbers[2]).await?;
        create_og_image(numbers[2], &title.to_uppercase(), &description, LilCamp)
      }
      _ => String::new(),
    };

    return Response::redirect(Url::parse(&*image)?);
  }

  Response::error("Bad Request", 400)
}

pub async fn handle_creation<D>(
  mut req: Request,
  _ctx: RouteContext<D>,
) -> worker::Result<Response> {
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
          numbers.push(segments[2].parse::<u32>().unwrap().try_into()?);
        } else {
          numbers.push(LilNouns as u64);
          numbers.push(Ethereum as u64);
          numbers.push(segments[1].parse::<u32>().unwrap().try_into()?);
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
          numbers.push(PropLot as u64);
          numbers.push(segments[1].parse::<u32>().unwrap().try_into()?);
        } else {
          return Response::error("Bad Request", 400);
        }

        Response::from_json(&UrlPayload {
          url: url.into(),
          sqid: Some(sqids.encode(&*numbers).unwrap()),
        })
      }
      Some("lilnouns.camp") | Some("www.lilnouns.camp") => {
        numbers.push(LilNouns as u64);

        let segments: Vec<_> = url
          .path_segments()
          .expect("Cannot get path segments")
          .filter(|segment| !segment.is_empty())
          .collect();

        if segments.is_empty() || segments[0] != "proposals" {
          return Response::error("Bad Request", 400);
        }

        if segments[0] == "proposals" {
          numbers.push(LilCamp as u64);
          numbers.push(segments[1].parse::<u32>().unwrap().try_into()?);
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
