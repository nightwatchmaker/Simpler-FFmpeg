# Simpler-FFmpeg

A small Rust CLI that makes common FFmpeg conversions less cursed.

## Usage

```bash
ffconvert "thing.mp4" "thing.mkv"
ffconvert "thing.mp4" "thing (2).mp4" --resolution 1920x1080 --fps 60
ffconvert input.mp4 output.mkv --audio-quality 3
ffconvert input.mp4 output.mkv --audio-copy
ffconvert input.mp4 frames/ --fps 12
ffconvert input.mp4 frames/ --fps 12 --image-format jpg
ffconvert input.mp4 output.mp4 --autoplay
ffconvert input.mp4 --onlyplay
```

Paths are passed directly to FFmpeg, so quoted spaces and parentheses work normally. If the input path is missing, ffconvert searches `$HOME` recursively by filename first, then searches the root filesystem (`/`) if needed and asks before selecting a match. Existing files are protected by default; use `--force` to replace one.

`--audio-quality` ranges from 1 to 10. Level 10 is normal high-quality audio. Lower levels progressively reduce sample rate and bit depth with FFmpeg's `aresample` and `acrusher` filters. `--audio-copy` skips audio processing entirely.

Video effects are numeric and can be combined. Normal values are usually `1` for saturation/contrast/gamma and `0` for brightness:

```bash
ffconvert input.mp4 output.mp4 --saturation 1.3 --contrast 1.1 --brightness 0.05 --sharpness 0.4
```

Available effects include `--saturation`, `--contrast`, `--brightness`, `--gamma`, `--hue`, `--sharpness`, `--blur`, `--noise`, `--grayscale`, and `--vignette`. They are chained into one FFmpeg video filter graph.

If the output ends in `/`, ffconvert extracts frames instead of making a video. Frames start at `1` and use `%d`-style numbering: `1.png`, `2.png`, `10.png`, and so on. PNG is the default; use `--image-format jpg`, `jpeg`, or `webp` for another format.

Use `--autoplay` to open a successfully converted video with `ffplay` after FFmpeg finishes. Frame extraction does not autoplay. Use `--onlyplay` when you want to skip conversion entirely and just open the input with ffplay:

```bash
ffconvert input.mp4 --onlyplay
```

## Detailed help

Use the general help or ask about one feature specifically. Both `thing` and `--thing` topic names work:

```bash
ffconvert --help
ffconvert --help --saturation
ffconvert --help effects
ffconvert --help audio-quality
```

## Build

```bash
cargo build --release
install -Dm755 target/release/ffconvert "$HOME/.local/bin/ffconvert"
```

Requires `ffmpeg` and `ffprobe` on `PATH`.
