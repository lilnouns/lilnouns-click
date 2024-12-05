use std::ops::Deref;

use axum::{
  extract::{Json, Path, State},
  http::{StatusCode, Uri},
  response::{Html, IntoResponse, Redirect},
};
use axum_cloudflare_adapter::wasm_compat;
use axum_macros::debug_handler;
use html_escape::encode_safe;
use html_minifier::minify;
use serde::{Deserialize, Serialize};
use sqids::Sqids;
use url::Url;
use worker::Env;

use crate::{
  queries::{fetch_lil_nouns_data, fetch_meta_gov_data, fetch_prop_lot_data},
  routes::{
    Community::LilNouns,
    Platform::{Ethereum, LilCamp, MetaGov, PropLot},
  },
  utils::create_og_image,
  AxumState,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct UrlPayload {
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

#[debug_handler]
#[wasm_compat]
pub(crate) async fn handle_redirect(
  State(state): State<AxumState>,
  Path(sqid): Path<String>,
  uri: Uri,
) -> impl IntoResponse {
  let env: &Env = state.env_wrapper.env.deref();

  let ga_id = env.secret("GA_ID").unwrap();
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
      let utm = format!(
        "utm_source=farcaster&utm_medium=social&utm_campaign=governor&utm_content=proposal_{}",
        numbers[2]
      );
      let url = format!("{}/{}?{}", "https://lilnouns.wtf/vote", numbers[2], utm);
      let (title, description) = fetch_lil_nouns_data(&env, numbers[2])
        .await
        .unwrap_or_default();
      let image = format!("{}/og.png", uri);
      (url, title, description, image)
    }
    (Some(LilNouns), Some(PropLot)) => {
      let utm = format!(
        "utm_source=farcaster&utm_medium=social&utm_campaign=proplot&utm_content=idea_{}",
        numbers[2]
      );
      let url = format!(
        "{}/{}?{}",
        "https://lilnouns.proplot.wtf/idea", numbers[2], utm
      );
      let (title, description) = fetch_prop_lot_data(&env, numbers[2])
        .await
        .unwrap_or_default();
      let image = format!("{}/og.png", uri);
      (url, title, description, image)
    }
    (Some(LilNouns), Some(MetaGov)) => {
      let utm = format!(
        "utm_source=farcaster&utm_medium=social&utm_campaign=metagov&utm_content=proposal_{}",
        numbers[2]
      );
      let url = format!(
        "{}/{}?{}",
        "https://lilnouns.wtf/vote/nounsdao", numbers[2], utm
      );
      let (title, description) = fetch_meta_gov_data(&env, numbers[2])
        .await
        .unwrap_or_default();
      let image = format!("{}/og.png", uri);
      (url, title, description, image)
    }
    (Some(LilNouns), Some(LilCamp)) => {
      let utm = format!(
        "utm_source=farcaster&utm_medium=social&utm_campaign=governor&tm_content=proposal_{}",
        numbers[2]
      );
      let url = format!(
        "{}/{}?{}",
        "https://lilnouns.camp/proposals", numbers[2], utm
      );
      let (title, description) = fetch_lil_nouns_data(&env, numbers[2])
        .await
        .unwrap_or_default();
      let image = format!("{}/og.png", uri);
      (url, title, description, image)
    }
    _ => (String::new(), String::new(), String::new(), String::new()),
  };

  let html_doc = format!(
    r#"
        <!DOCTYPE html>
        <html lang="en">
        <head>
            <meta charset="UTF-8">
            <meta name="viewport" content="width=device-width, initial-scale=1.0">

            <meta property="og:site_name" content="Lil Nouns">
            <meta property="og:url" content="{}">
            <meta property="og:type" content="website">
            <meta property="og:title" content="{}">
            <meta property="og:description" content="{}">

            <meta property="og:image" content="{}">
            <meta property="og:image:secure_url" content="{}" />
            <meta property="og:image:type" content="image/png" />
            <meta property="og:image:width" content="1200" />
            <meta property="og:image:height" content="630" />
            <meta property="og:image:alt" content="{}" />

            <meta name="twitter:card" content="summary_large_image">
            <meta property="twitter:domain" content="lilnouns.click">
            <meta property="twitter:url" content="{}">
            <meta name="twitter:title" content="{}">
            <meta name="twitter:description" content="{}">
            <meta name="twitter:image" content="{}">

            <meta property="fc:frame" content="vNext" />
            <meta property="fc:frame:image" content="{}" />

            <meta http-equiv="refresh" content="3; url={}" />

            <title>{}</title>
            <meta name="description" content="{}">

            <link rel="preconnect" href="https://fonts.googleapis.com">
            <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
            <link href="https://fonts.googleapis.com/css2?family=Londrina+Solid:wght@100;300;400;900&display=swap" rel="stylesheet">

            <!-- Google tag (gtag.js) -->
            <script async src="https://www.googletagmanager.com/gtag/js?id={}"></script>
            <script>
              window.dataLayer = window.dataLayer || [];
              function gtag(){{dataLayer.push(arguments);}}
              gtag('js', new Date());

              gtag('config', '{}');
            </script>
        </head>
        <body style="margin: 0; padding: 0; display: flex; justify-content: center; align-items: center; height: 100vh; background-color: #f0f0f0; font-family: 'Londrina Solid', cursive;">
            <div style="text-align: center;">
                <div style="padding: 20px;">
                    <img src="https://lilnouns.wtf/static/media/lil-loading-skull.b7a846e1.gif" alt="Loading Skull" style="width: 192px; height: 192px;">
                    <p style="margin-top: 10px; font-size: 24px; font-weight: bold;">Redirecting...</p>
                    <p><a style="font-size: 24px; text-decoration: none;" href="{}">{}</a></p>
                </div>
            </div>
        </body>
        </html>
    "#,
    url,                       // OpenGraph URL
    encode_safe(&title),       // OpenGraph Title
    encode_safe(&description), // OpenGraph Description
    image,                     // OpenGraph Image URL
    image,                     // OpenGraph Image Secure URL
    encode_safe(&title),       // OpenGraph Image Alt
    url,                       // Twitter URL
    encode_safe(&title),       // Twitter Title
    encode_safe(&description), // Twitter Description
    image,                     // Twitter Image
    image,                     // Farcaster Image
    url,                       // Page Refresh URL
    encode_safe(&title),       // Page Title
    encode_safe(&description), // Page Description
    ga_id,                     // Google Analytics ID
    ga_id,                     // Google Analytics ID
    url,                       // Page Content Link URL
    encode_safe(&title),       // Page Content Link Title
  );

  let minified_html = minify(html_doc).expect("Failed to minify HTML");

  Html(minified_html)
}

#[debug_handler]
#[wasm_compat]
pub(crate) async fn handle_og_image(
  State(state): State<AxumState>,
  Path(sqid): Path<String>,
) -> impl IntoResponse {
  let env: &Env = state.env_wrapper.env.deref();

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
      let (title, description) = fetch_lil_nouns_data(&env, numbers[2])
        .await
        .unwrap_or_default();
      create_og_image(numbers[2], &title.to_uppercase(), &description, Ethereum)
    }
    (Some(LilNouns), Some(PropLot)) => {
      let (title, description) = fetch_prop_lot_data(&env, numbers[2])
        .await
        .unwrap_or_default();
      create_og_image(numbers[2], &title.to_uppercase(), &description, PropLot)
    }
    (Some(LilNouns), Some(MetaGov)) => {
      let (title, description) = fetch_meta_gov_data(&env, numbers[2])
        .await
        .unwrap_or_default();
      create_og_image(numbers[2], &title.to_uppercase(), &description, MetaGov)
    }
    (Some(LilNouns), Some(LilCamp)) => {
      let (title, description) = fetch_lil_nouns_data(&env, numbers[2])
        .await
        .unwrap_or_default();
      create_og_image(numbers[2], &title.to_uppercase(), &description, LilCamp)
    }
    _ => String::new(),
  };

  let _ = Redirect::temporary(image.as_str());
}

#[debug_handler]
#[wasm_compat]
pub(crate) async fn handle_creation(Json(payload): Json<UrlPayload>) -> impl IntoResponse {
  let sqids = Sqids::default();
  let mut numbers: Vec<u64> = Vec::new();

  match Url::parse(&payload.url) {
    Ok(url) => match url.host_str() {
      Some("lilnouns.wtf") | Some("www.lilnouns.wtf") => {
        let segments: Vec<_> = url
          .path_segments()
          .map(|segments| segments.filter(|s| !s.is_empty()).collect())
          .unwrap_or_else(Vec::new);

        if segments.is_empty() || segments[0] != "vote" {
          return (StatusCode::BAD_REQUEST, "Bad Request").into_response();
        }

        if segments[1] == "nounsdao" {
          numbers.push(LilNouns as u64);
          numbers.push(MetaGov as u64);
          numbers.push(
            segments[2]
              .parse::<u32>()
              .unwrap()
              .try_into()
              .expect("Failed to convert number"),
          );
        } else {
          numbers.push(LilNouns as u64);
          numbers.push(Ethereum as u64);
          numbers.push(
            segments[1]
              .parse::<u32>()
              .unwrap()
              .try_into()
              .expect("Failed to convert number"),
          );
        }

        Json(UrlPayload {
          url: payload.url.clone(),
          sqid: Some(sqids.encode(&numbers).unwrap()),
        })
        .into_response()
      }
      Some("lilnouns.proplot.wtf") | Some("www.lilnouns.proplot.wtf") => {
        numbers.push(LilNouns as u64);

        let segments: Vec<_> = url
          .path_segments()
          .map(|segments| segments.filter(|s| !s.is_empty()).collect())
          .unwrap_or_else(Vec::new);

        if segments.is_empty() || segments[0] != "idea" {
          return (StatusCode::BAD_REQUEST, "Bad Request").into_response();
        }

        numbers.push(PropLot as u64);
        numbers.push(
          segments[1]
            .parse::<u32>()
            .unwrap()
            .try_into()
            .expect("Failed to convert number"),
        );

        Json(UrlPayload {
          url: payload.url.clone(),
          sqid: Some(sqids.encode(&numbers).unwrap()),
        })
        .into_response()
      }
      Some("lilnouns.camp") | Some("www.lilnouns.camp") => {
        numbers.push(LilNouns as u64);

        let segments: Vec<_> = url
          .path_segments()
          .map(|segments| segments.filter(|s| !s.is_empty()).collect())
          .unwrap_or_else(Vec::new);

        if segments.is_empty() || segments[0] != "proposals" {
          return (StatusCode::BAD_REQUEST, "Bad Request").into_response();
        }

        numbers.push(LilCamp as u64);
        numbers.push(
          segments[1]
            .parse::<u32>()
            .unwrap()
            .try_into()
            .expect("Failed to convert number"),
        );

        Json(UrlPayload {
          url: payload.url.clone(),
          sqid: Some(sqids.encode(&numbers).unwrap()),
        })
        .into_response()
      }
      _ => (StatusCode::BAD_REQUEST, "Bad Request").into_response(),
    },
    Err(_) => (StatusCode::BAD_REQUEST, "Invalid URL").into_response(),
  }
}
