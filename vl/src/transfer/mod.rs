use anyhow::Result;
use std::process::Command;

fn vec_to_string(v: Vec<u8>) -> String {
    String::from_utf8(v).unwrap().trim().to_string()
}

fn excute(args: &[&str]) -> Result<String> {
    let out = Command::new(args[0]).args(&args[1..]).output()?;
    match out.status.success() {
        true => Ok(vec_to_string(out.stdout)),
        false => Err(anyhow::Error::msg(format!(
            "Command not successful. This is stderr:\n{}",
            vec_to_string(out.stderr)
        ))),
    }
}

#[cfg(target_os = "windows")]
static FFMPET: &str = "./ffmpeg.exe";

#[cfg(target_os = "linux")]
static FFMPET: &str = "./ffmpeg";

pub async fn run(
    source: &str,
    output: &str,
    pic: Option<&str>,
    extension: &str,
    title: &str,
    artist: &str,
) -> Result<()> {
    let title = &format!("title={title}");
    let artist = &format!("artist={artist}");

    let mut input_arg = vec![FFMPET, "-i", source];

    if pic.is_some() {
        input_arg.append(&mut vec!["-i", pic.unwrap()]);
    }

    input_arg.append(&mut vec!["-metadata", title]);
    input_arg.append(&mut vec!["-metadata", artist]);

    match extension {
        "flac" => input_arg.append(&mut vec!["-acodec", "flac"]),
        "m4a" => input_arg.append(&mut vec!["-c", "copy"]),
        _ => panic!("Should not happened!"),
    }

    if pic.is_some() {
        input_arg.append(&mut vec![
            "-c:v:1",
            "jpg",
            "-disposition:v:0",
            "attached_pic",
        ]);
    }

    input_arg.push(output);

    excute(&input_arg)?;

    Ok(())
}
