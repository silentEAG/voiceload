use std::path::PathBuf;
use std::time::Instant;
use anyhow::Result;
use log::error;
use log::info;
use reqwest::header;
use reqwest::header::HeaderMap;
use reqwest::header::HeaderValue;
use tokio::join;
use tokio::sync::mpsc;

use parse::CONFIG;
use tokio::sync::mpsc::Sender;
use vl::catcher::view;
use vl::catcher::link;
use vl::loader::load;
use vl::transfer;

use crate::parse::SESSION;
use crate::util::safe_filename;

mod config;
mod parse;
mod util;

#[derive(PartialEq)]
enum Audio {
    Flac,
    MP4
}

async fn run_one_by_one(index: usize, id: &str, tx: Sender<Context>) -> Result<()> {

   // Get audio information.
   info!("[{index}] Get information about {}", id);
   let view = view::api(id)
   .await?;

   // Get audio link.
   let link = link::api(&view.bvid, view.pages[0].cid, 16 | 256, Some(SESSION.clone()))
       .await?;
   
   // Download audio.
   info!("[{index}] Downloading {}", id);
   // If ids.len() > 1, you should not to set filename.
   let filename = match !CONFIG.filename().is_empty() {
       true => CONFIG.filename(),
       false => view.title.to_string()
   };
   let filename = safe_filename(&filename);

   // Set default audio type.
   let mut audio_type = Audio::MP4;
   let mut load_url = link.dash.audio[0].base_url.to_string();

   if CONFIG.flac_allowed() {
        if let Some(flac) = link.dash.flac {
            load_url = flac.audio.unwrap().base_url;
            audio_type = Audio::Flac;
        }
   }

   load(&load_url, &filename, &CONFIG.path()).await?;

   // Preparing for transform audio.
   let context = Context {
        index,
        audio: audio_type,
        filename: filename.to_string(),
        owner: view.owner.name,
    };

    tokio::spawn(async move {
        let _ = tx.send(context).await;
    });

    Ok(())
}

struct Context {
    index: usize,
    audio: Audio,
    filename: String,
    owner: String
}

pub async fn run() {

    let id = CONFIG.id();
    let ids: Vec<&str> = id.split(' ').collect();

    let (tx, mut rx) = mpsc::channel::<Context>(ids.len());

    let transform_handler = tokio::spawn(async move {
        while let Some(context) = rx.recv().await {

            info!("[{}] starting transform '{}'", context.index, context.filename);

            let extension = match context.audio {
                Audio::MP4 => "mp4",
                Audio::Flac => "flac"
            };

            let path = PathBuf::from(CONFIG.path()).join(&context.filename);

            let source = path.with_extension("m4s");
            let output = path.with_extension(extension);

            if !source.exists() {
                error!("[{}] Source file not exists.", context.index);
                let _ = std::fs::remove_file(source);
                continue;
            }

            if output.exists() {
                error!("[{}] Output file already exists.", context.index);
                let _ = std::fs::remove_file(source);
                continue;
            }

            if let Err(e) = transfer::run(
                source.to_str().unwrap(),
                output.to_str().unwrap(),
                extension, &context.filename, &context.owner).await {
                    error!("[{}] {e}", context.index);
            }

            let _ = std::fs::remove_file(source);
            info!("[{}] finishing transforming '{}'", context.index, context.filename);
        };
    });

    for (index, id) in ids.iter().enumerate() {

        info!("[{index}] id = {id} starting to work");

        if let Err(e) = run_one_by_one(index, id, tx.clone()).await {
            error!("[{index}] Error occurs when viewing or downloading audio: {}", e);
        }

        info!("[{index}] id = {id} finishing download");
    }
    drop(tx);
    
    let _ = join!(transform_handler);
    
}

fn summary() {

}

#[tokio::main]
async fn main() {
    let total_cost = Instant::now();
    simple_logger::init_with_level(log::Level::Debug).unwrap();
    // let args = Args::parse();
    run().await;
    
}
