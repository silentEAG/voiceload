


use clap::Parser;
use once_cell::sync::Lazy;
use reqwest::header::{HeaderMap, self, HeaderValue};

use crate::config::{ConfigBuilder, ConfigItems};

#[derive(Parser, Debug)]
#[command(name = "bili-voiceload", author, version, about, long_about = None)]
pub struct Args {

    /// Aid/Bvids to download, can be multiple
    #[arg(short, long)]
    id: Vec<String>,
   
    /// Allow downloading flac [default: false]
    #[arg(short = 'F', long)]
    flac_allowed: Option<bool>,

    /// Allow downloading dolby [default: false]
    #[arg(short = 'D', long)]
    dolby_allowed: Option<bool>,

    /// Allow adding picture to audio [default: false]
    #[arg(short = 'P', long)]
    picture_allowed: Option<bool>,

    /// Path to save audio files, [default: ./]
    #[arg(short, long)]
    path: Option<String>,

    /// (Optional) Filename to save [default: the title of the audio]
    #[arg(short = 'o')]
    filename: Option<String>,

    /// (Optional) Sessiondata for login aiming to dolby or flac [default: None]
    #[arg(short, long)]
    session: Option<String>,

    /// (Optional) Config file path
    #[arg(short, long, default_value = "./config.json")]
    config: String
}

pub static CONFIG: Lazy<ConfigItems> = Lazy::new(|| {
    let args = Args::parse();

    if args.id.is_empty() {
        panic!("id must be input, try -h/--help for help");
    }

    ConfigBuilder::default()
        // Parsing file
        .add_file(&args.config)
        // Parsing args   
        .dolby_allowed(args.dolby_allowed)
        .flac_allowed(args.flac_allowed)
        .pic_allowed(args.picture_allowed)
        .filename(args.filename)
        .path(args.path)
        .session(args.session)
        .id(Some(args.id.join(" ")))
        .build()
});

pub static SESSION: Lazy<HeaderMap> = Lazy::new(|| {
    let mut headers = HeaderMap::new();
    headers.insert(header::COOKIE, HeaderValue::from_str(&format!("SESSDATA={}", CONFIG.session())).unwrap());
    headers
});