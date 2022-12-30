
use std::process::Command;
use anyhow::Result;

fn vec_to_string(v: Vec<u8>) -> String {
    String::from_utf8(v).unwrap().trim().to_string()
}

fn excute(args: &[&str]) -> Result<String> {
    let out = Command::new(args[0]).args(&args[1..]).output()?;
    match out.status.success() {
        true => Ok(vec_to_string(out.stdout)),
        false => Err(anyhow::Error::msg(format!(
            "Command not successful. This is stderr:\n{}",
            vec_to_string(out.stderr)))),
    }
}

#[cfg(target_os = "windows")]
static FFMPET: &str = "./ffmpeg.exe";

#[cfg(target_os = "linux")]
static FFMPET: &str = "./ffmpeg";

pub async fn run(source: &str, output: &str, extension: &str, title: &str, artist: &str) -> Result<()> {
    let title = &format!("title=\"{title}\"");
    let artist = &format!("artist=\"{artist}\"");
    match extension {
        "flac" => excute(&[FFMPET, "-i", source, "-metadata", title, "-metadata", artist, "-acodec", "flac", output])?,
        "mp4" => excute(&[FFMPET, "-i", source, "-metadata", title, "-metadata", artist, "-codec", "copy", output])?,
        _ => panic!("Should not happened!")
    };
    Ok(())
}
