use anyhow::Result;
use log::error;
use log::info;
use log::warn;
use std::path::PathBuf;
use std::time::Duration;
use std::time::Instant;
use tokio::join;
use tokio::runtime;
use tokio::sync::mpsc;

use parse::CONFIG;
use tokio::sync::mpsc::Sender;
use vl::catcher::link;
use vl::catcher::view;
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
    M4a,
}

async fn run_one_by_one(index: usize, id: &str, tx: Sender<Context>) -> Result<()> {
    // Get audio information.
    info!("[{index}] Get information about {}", id);
    let view = view::api(id).await?;

    // Get audio link.
    let link = link::api(
        &view.bvid,
        view.pages[0].cid,
        16 | 256,
        Some(SESSION.clone()),
    )
    .await?;

    // Download audio.
    info!("[{index}] Downloading {}", id);
    // If ids.len() > 1, you should not to set filename.
    let filename = match !CONFIG.filename().is_empty() {
        true => CONFIG.filename(),
        false => view.title.to_string(),
    };
    let filename = safe_filename(&filename);

    // Set default audio type.
    let mut audio_type = Audio::M4a;
    let mut load_url = link.dash.audio[0].base_url.to_string();

    if CONFIG.flac_allowed() {
        if let Some(flac) = link.dash.flac {
            load_url = flac.audio.unwrap().base_url;
            audio_type = Audio::Flac;
        }
    }

    load(&load_url, &filename, &CONFIG.path(), "m4s").await?;

    if CONFIG.pic_allowed() {
        load(&view.pic, &filename, &CONFIG.path(), "jpg").await?;
    }

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
    owner: String,
}

pub async fn run() {
    let total_cost = Instant::now();

    let ids = CONFIG.id();

    let (tx, mut rx) = mpsc::channel::<Context>(ids.len());

    let transform_handler = tokio::spawn(async move {
        while let Some(context) = rx.recv().await {
            info!(
                "[{}] Starting transform '{}'",
                context.index, context.filename
            );

            let extension = match context.audio {
                Audio::M4a => "m4a",
                Audio::Flac => "flac",
            };

            let path = PathBuf::from(CONFIG.path()).join(&context.filename);

            let source = path.with_extension("m4s");
            let output = path.with_extension(extension);

            if !source.exists() {
                error!("[{}] Source file not exists.", context.index);
                let _ = std::fs::remove_file(source.clone());
                if CONFIG.pic_allowed() {
                    let _ = std::fs::remove_file(source.with_extension("jpg"));
                }
                continue;
            }

            if output.exists() {
                error!("[{}] Output file already exists.", context.index);
                let _ = std::fs::remove_file(source.clone());
                if CONFIG.pic_allowed() {
                    let _ = std::fs::remove_file(source.with_extension("jpg"));
                }
                continue;
            }

            let pic = source.with_extension("jpg");
            let pic = match CONFIG.pic_allowed() {
                true => pic.to_str(),
                false => None,
            };

            if let Err(e) = transfer::run(
                source.to_str().unwrap(),
                output.to_str().unwrap(),
                pic,
                extension,
                &context.filename,
                &context.owner,
            )
            .await
            {
                error!("[{}] {e}", context.index);
            }

            let _ = std::fs::remove_file(source.clone());
            if CONFIG.pic_allowed() {
                let _ = std::fs::remove_file(source.with_extension("jpg"));
            }
            info!(
                "[{}] Finish transforming '{}'",
                context.index, context.filename
            );
        }
    });

    for (index, id) in ids.iter().enumerate() {
        let index = index + 1;

        info!("[{index}] id = {id} starting to work");

        if let Err(e) = run_one_by_one(index, id, tx.clone()).await {
            error!(
                "[{index}] Error occurs when viewing or downloading audio: {}",
                e.source().unwrap()
            );
        }

        info!("[{index}] id = {id} finish download");
    }
    drop(tx);

    let _ = join!(transform_handler);

    summary(total_cost.elapsed());
}

fn summary(cost: Duration) {
    info!("Total costs: {:?}", cost);
}

fn pre_work() {
    // TODO: 添加session有效性验证
    match CONFIG.session().is_empty() {
        true => warn!("You are not set account session, so can't download flac/dolby"),
        false => info!("You have already set account session, can download flac/dolby if supports"),
    };
}

fn main() {
    simple_logger::init_with_level(log::Level::Info).unwrap();
    pre_work();
    let runtime = runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();
    runtime.block_on(run());
}
