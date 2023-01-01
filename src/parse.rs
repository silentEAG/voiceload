use clap::Parser;
use log::{error, info};
use once_cell::sync::Lazy;
use reqwest::{
    blocking::{self, Client},
    header::{self, HeaderMap, HeaderValue},
    redirect::Policy,
    IntoUrl, Url,
};

use crate::{
    config::{ConfigBuilder, ConfigItems},
    util::{is_id, is_link, read_file_string},
};

#[derive(Parser, Debug)]
#[command(name = "bili-voiceload", author, version, about, long_about = None)]
pub struct Args {
    /// aid/bvid/link to download, can be multiple
    #[arg(short, long)]
    inputs: Option<Vec<String>>,

    /// parsing a file content line by line to get input, split by '\n'
    #[arg(short, long)]
    file_input: Option<String>,

    /// Allow downloading flac [default: false]
    #[arg(short = 'F', long)]
    flac_allowed: Option<bool>,

    /// Allow downloading dolby [default: false]
    #[arg(short = 'D', long)]
    dolby_allowed: Option<bool>,

    /// Allow adding picture to audio [default: false]
    #[arg(short = 'P', long)]
    picture_allowed: Option<bool>,

    /// Path to save audio files [default: ./]
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
    config: String,
}

fn error_input() -> ! {
    panic!("\"inputs\" and \"file input\" are all empty, just add at least one of them to run, or try -h/--help for help");
}

fn get_bv<U>(link: U) -> Option<String>
where
    U: IntoUrl,
{
    let link = link.into_url();
    if link.is_err() {
        return None;
    }
    match link.unwrap().path_segments() {
        Some(mut paths) => paths.find(|id| is_id(id)).map(|id| id.to_string()),
        None => None,
    }
}

static PARSE_CILENT: Lazy<Client> = Lazy::new(|| {
    blocking::Client::builder()
        .redirect(Policy::none())
        .build()
        .unwrap()
});

fn parse_link(link: Url) -> Option<String> {
    match link.host_str() {
        Some("www.bilibili.com") => get_bv(link),
        Some("b23.tv") => {
            match PARSE_CILENT.get(link).send().map(|response| {
                let headers = response.headers();
                if let Some(link_str) = headers.get(header::LOCATION) {
                    let link_str = link_str.to_str().unwrap();
                    get_bv(link_str)
                } else {
                    None
                }
            }) {
                Ok(res) => res,
                Err(_) => None,
            }
        }
        _ => None,
    }
}

pub static CONFIG: Lazy<ConfigItems> = Lazy::new(|| {
    let args = Args::parse();

    if args.inputs.is_none() && args.file_input.is_none() {
        error_input();
    }

    let mut pre_inputs = match args.inputs {
        Some(arg_inputs) => arg_inputs,
        None => Vec::new(),
    };

    if let Some(file_input) = args.file_input {
        info!("Starting to get file input");
        let file_inputs = match read_file_string(&file_input) {
            Ok(content) => content
                .split('\n')
                .filter(|s| !s.is_empty())
                .map(|s| s.trim().to_owned())
                .collect(),
            Err(e) => {
                error!("{e}");
                vec![]
            }
        };
        pre_inputs.extend(file_inputs);
    };

    let pre_cnt = pre_inputs.len();
    info!("Input total: {pre_cnt}");
    pre_inputs.is_empty().then(error_input);

    info!("Starting to parse inputs");

    let mut res_inputs = Vec::<String>::new();

    for pre_input in pre_inputs {
        match is_id(&pre_input) {
            true => res_inputs.push(pre_input),
            false => {
                if let Ok(link) = is_link(&pre_input) {
                    if link.scheme() == "http" || link.scheme() == "https" {
                        if let Some(res) = parse_link(link) {
                            res_inputs.push(res);
                            continue;
                        }
                    }
                }
                error!("Parsing {} failed, skip it", pre_input);
            }
        };
    }

    let res_cnt = res_inputs.len();
    info!("Succeed to parse: {res_cnt}");
    res_inputs.is_empty().then(error_input);

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
        .id(Some(res_inputs))
        .build()
});

pub static SESSION: Lazy<HeaderMap> = Lazy::new(|| {
    let mut headers = HeaderMap::new();
    headers.insert(
        header::COOKIE,
        HeaderValue::from_str(&format!("SESSDATA={}", CONFIG.session())).unwrap(),
    );
    headers
});

#[test]
fn test_parse_url() {
    let url = Url::parse("https://www.bilibili.com/a/b/c").unwrap();
    println!("{:?}", url.host_str());
    println!("{:?}", url.path_segments().unwrap().collect::<Vec<&str>>());
}
