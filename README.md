# Bili-voiceload

```text
A simple cli tool for downloading audio in bilibili.

Usage: voiceload.exe [OPTIONS]

Options:
  -i, --id <ID>
          Aid/Bvids to download, can be multiple
  -F, --flac-allowed <FLAC_ALLOWED>
          Allow downloading flac, false as default [possible values: true, false]
  -D, --dolby-allowed <DOLBY_ALLOWED>
          Allow downloading dolby, false as default [possible values: true, false]
  -P, --picture-allowed <PICTURE_ALLOWED>
          Allow adding picture to audio, false as default [possible values: true, false]
  -p, --path <PATH>
          Path to save audio files, current dir as default
  -o <FILENAME>
          (Optional) Filename to save. the title of audio as default
  -s, --session <SESSION>
          (Optional) Sessiondata for login aiming to dolby or flac
  -h, --help
          Print help information
  -V, --version
          Print version information
```