use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
use regex::Regex;
use unidecode::unidecode;

use crate::handlers::Platform;

pub fn truncate_and_clean_string(input: &str, limit: usize) -> String {
  // Check if the first line contains "#"
  let mut lines = input.lines();
  let first_line = lines.next().unwrap_or("");
  let mut remaining_text = if first_line.contains("#") {
    lines.collect::<Vec<&str>>().join("\n")
  } else {
    input.to_string()
  };

  // Remove HTML tags
  let tag_re = Regex::new(r"<[^>]*>").unwrap();
  remaining_text = tag_re.replace_all(&remaining_text, "").to_string();

  // Remove markdown tags
  let markdown_re = Regex::new(r"\*{1,2}|_{1,2}|#{1,6}|\`{1,3}|\[(.*?)\]\((.*?)\)").unwrap();
  remaining_text = markdown_re.replace_all(&remaining_text, "$1").to_string();

  // Remove URLs
  let url_re = Regex::new(r"https?://\S*").unwrap();
  remaining_text = url_re.replace_all(&remaining_text, "").to_string();

  // Replace new lines and double spaces with single space
  let new_line_re = Regex::new(r"\n+").unwrap();
  remaining_text = new_line_re.replace_all(&remaining_text, " ").to_string();
  let space_re = Regex::new(r" {2,}").unwrap();
  remaining_text = space_re.replace_all(&remaining_text, " ").to_string();

  // Remove all non-alphabet characters from the start of the string
  let start_re = Regex::new(r"^[^a-zA-Z]*").unwrap();
  remaining_text = start_re.replace(&remaining_text, "").to_string();

  // Transliterate non-ASCII characters to closest ASCII
  let remaining_text = unidecode(&remaining_text);

  // Remove non-ASCII characters
  let non_ascii_re = Regex::new(r"[^\x00-\x7F]").unwrap();
  let remaining_text = non_ascii_re.replace_all(&remaining_text, "").to_string();

  // Truncate and add ... if longer than limit
  if remaining_text.chars().count() > limit {
    let mut truncated = remaining_text.chars().take(limit).collect::<String>();
    truncated.push_str("...");
    truncated
  } else {
    remaining_text
  }
}

pub fn create_og_image(id: u64, title: &str, description: &str, platform: Platform) -> String {
  let non_alpha_numeric = Regex::new("[^a-zA-Z0-9 .:]").unwrap();

  let title = non_alpha_numeric.replace_all(title, "");
  let description = non_alpha_numeric.replace_all(description, "");

  let logo_image = format!(
    "/{}",
    match platform {
      Platform::Ethereum => "l_lil_noun_logo",
      Platform::PropLot => "l_lil_noun_logo",
      Platform::MetaGov => "l_lil_noun_logo",
      Platform::LilCamp => "l_lil_noun_logo",
    }
  );
  let foreground_color = format!(
    "co_rgb:{}",
    match platform {
      Platform::Ethereum => "FFFFFF",
      Platform::PropLot => "000000",
      Platform::MetaGov => "FFFFFF",
      Platform::LilCamp => "FFFFFF",
    }
  );
  let background_color = format!(
    "/b_rgb:{}",
    match platform {
      Platform::Ethereum => "7DC4F2",
      Platform::PropLot => "FFEF2E",
      Platform::MetaGov => "F0C850",
      Platform::LilCamp => "156636",
    }
  );

  let cloudinary_id = "nekofar";
  let cloudinary_url = format!("https://res.cloudinary.com/{}/image/upload", cloudinary_id);

  let title_encoded = format!(
    "/l_text:{}_64:{},{},c_fit,w_1100",
    utf8_percent_encode("LondrinaSolid-Regular.ttf", NON_ALPHANUMERIC),
    utf8_percent_encode(&title, NON_ALPHANUMERIC),
    &foreground_color
  );

  let description_encoded = format!(
    "/l_text:{}_48:{},{},c_fit,w_1050",
    utf8_percent_encode("LondrinaSolid-Light.ttf", NON_ALPHANUMERIC),
    utf8_percent_encode(&description, NON_ALPHANUMERIC),
    &foreground_color
  );

  let reference_number_formatted = format!(
    "/l_text:{}_56:{},{},c_fit,w_1050",
    utf8_percent_encode("LondrinaSolid-Black.ttf", NON_ALPHANUMERIC),
    utf8_percent_encode(format!("Prop {}", id).as_str(), NON_ALPHANUMERIC),
    &foreground_color
  );

  let parts = vec![
    &cloudinary_url,
    &background_color,
    "/c_scale,h_630,w_1200",
    &logo_image,
    "/c_scale,w_210/e_screen,fl_layer_apply,g_north_west,x_70,y_70",
    &title_encoded,
    "/fl_layer_apply,g_north_west,x_70,y_220",
    &description_encoded,
    "/fl_layer_apply,g_north_west,x_70,y_385",
    &reference_number_formatted,
    "/fl_layer_apply,g_north_east,x_70,y_70",
    "/f_auto,q_auto:eco",
    "/blank.png",
  ];

  parts.join("")
}
