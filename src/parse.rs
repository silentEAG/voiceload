
use clap::Parser;
use once_cell::sync::Lazy;
use reqwest::header::{HeaderMap, self, HeaderValue};

use crate::config::{ConfigBuilder, ConfigItems};

#[derive(Parser, Debug)]
#[command(name = "bili-voiceload", author, version, about, long_about = None)]
pub struct Args {

    /// Aid/Bvids to download, space for split.
    #[arg(short, long)]
    id: Vec<String>,
   
    /// Allow downloading flac, false as default.
    #[arg(short = 'F', long)]
    flac_allowed: Option<bool>,

    /// Allow downloading dolby, false as default.
    #[arg(short = 'D', long)]
    dolby_allowed: Option<bool>,

    /// Path to save audio files, current dir as default.
    #[arg(short, long)]
    path: Option<String>,

    /// (Optional) Filename to save. the title of audio as default.
    #[arg(short = 'o')]
    filename: Option<String>,

    /// (Optional) Sessiondata for login aiming to dolby or flac.
    #[arg(short, long)]
    session: Option<String>
}

pub static CONFIG: Lazy<ConfigItems> = Lazy::new(|| {
    let args = Args::parse();

    if args.id.is_empty() {
        panic!("id must be input, try -h/--help for help");
    }

    ConfigBuilder::default()
        // Parsing file
        // .add_file("./config.json")
        // Parsing args
        .dolby_allowed(args.dolby_allowed)
        .flac_allowed(args.flac_allowed)
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