use std::{
    env, fs,
    io::{self, IsTerminal, Read, Write},
    path::Path,
    process::{Command, ExitCode, Stdio},
};

#[derive(Debug)]
struct Options {
    input: String,
    output: String,
    resolution: Option<String>,
    fps: Option<String>,
    video_filters: Vec<String>,
    image_format: String,
    audio_quality: u8,
    audio_copy: bool,
    force: bool,
}

fn print_topic_help(topic: &str) {
    let topic = topic.strip_prefix("--").unwrap_or(topic);
    let text = match topic {
        "resolution" => {
            r#"--resolution WIDTHxHEIGHT

Scales the video while preserving its aspect ratio. The requested size is
used as a maximum, so a video will not be stretched or distorted.

Example:
  ffconvert input.mp4 output.mp4 --resolution 1920x1080"#
        }
        "fps" => {
            r#"--fps FPS

Sets the output frame rate. Fractional values are allowed, such as 29.97.
This can duplicate or drop frames when converting from another frame rate.

Example:
  ffconvert input.mp4 output.mp4 --fps 60"#
        }
        "saturation" => {
            r#"--saturation NUMBER

Controls color intensity. 1 is normal, 0 removes all color, and values above
1 make colors stronger.

Example:
  ffconvert input.mp4 output.mp4 --saturation 1.3"#
        }
        "contrast" => {
            r#"--contrast NUMBER

Controls the difference between dark and bright areas. 1 is normal, 0 makes
the image flat gray, and values above 1 increase contrast.

Example:
  ffconvert input.mp4 output.mp4 --contrast 1.2"#
        }
        "brightness" => {
            r#"--brightness NUMBER

Adjusts overall brightness. 0 is normal. Values from -1 to 1 are accepted.
Negative values darken the video and positive values brighten it.

Example:
  ffconvert input.mp4 output.mp4 --brightness 0.1"#
        }
        "gamma" => {
            r#"--gamma NUMBER

Adjusts midtones without changing brightness as directly as --brightness.
1 is normal. Values below 1 darken midtones and values above 1 brighten them.

Example:
  ffconvert input.mp4 output.mp4 --gamma 1.1"#
        }
        "hue" => {
            r#"--hue DEGREES

Rotates the colors around the hue wheel. 0 leaves colors unchanged.

Example:
  ffconvert input.mp4 output.mp4 --hue 30"#
        }
        "sharpness" => {
            r#"--sharpness NUMBER

Adds sharpening. 0 is off; higher values make edges more pronounced.
Use small values such as 0.3 or 0.5 to avoid ugly halos.

Example:
  ffconvert input.mp4 output.mp4 --sharpness 0.4"#
        }
        "blur" => {
            r#"--blur NUMBER

Adds box blur. 0 is off. Larger values produce a stronger blur.

Example:
  ffconvert input.mp4 output.mp4 --blur 2"#
        }
        "noise" => {
            r#"--noise NUMBER

Adds animated video grain/noise. 0 is off. Larger values are more noticeable.

Example:
  ffconvert input.mp4 output.mp4 --noise 8"#
        }
        "grayscale" => {
            r#"--grayscale NUMBER

Desaturates the video. 0 keeps the original color and 1 is fully grayscale.
Values between 0 and 1 create a partial desaturation effect.

Example:
  ffconvert input.mp4 output.mp4 --grayscale 1"#
        }
        "vignette" => {
            r#"--vignette NUMBER

Darkens the corners of the video. The value is the vignette angle in radians.
0 is off. A common starting value is 0.5.

Example:
  ffconvert input.mp4 output.mp4 --vignette 0.5"#
        }
        "audio-quality" => {
            r#"--audio-quality 1-10

Controls intentional audio degradation. 10 is normal high-quality audio.
Lower values reduce sample rate and bit depth, eventually creating lo-fi or
bitcrushed audio. This requires audio re-encoding.

Use --audio-copy instead when you want the original audio stream copied.

Example:
  ffconvert input.mp4 output.mp4 --audio-quality 4"#
        }
        "audio-copy" => {
            r#"--audio-copy

Copies the input audio stream without filtering or re-encoding it. This is
faster and avoids additional quality loss, but it cannot be combined with
--audio-quality below 10.

Example:
  ffconvert input.mp4 output.mkv --audio-copy"#
        }
        "force" | "overwrite" => {
            r#"--force

Overwrites an existing output file without asking. Without --force,
ffconvert asks "Overwrite it? [Y/N]" before replacing an existing file.

Example:
  ffconvert input.mp4 output.mp4 --force"#
        }
        "search" | "missing-file" => {
            r#"Missing input search

If the input path is not found, ffconvert searches your home directory
recursively by filename. Each match is shown by folder and confirmed with a
Y/N prompt before conversion starts.

Example:
  ffconvert recording.mp4 output.mkv"#
        }
        "effects" => {
            r#"Video effects

These numeric effects can be combined in one command:
  --saturation, --contrast, --brightness, --gamma, --hue, --sharpness,
  --blur, --noise, --grayscale, and --vignette.

Example:
  ffconvert input.mp4 output.mp4 --saturation 1.2 --contrast 1.1 --hue 10"#
        }
        _ => {
            eprintln!("Unknown help topic: {topic}");
            if let Some(suggestion) = closest_option(topic) {
                eprintln!("Did you mean ffconvert --help {suggestion}?");
            } else {
                eprintln!("Try: ffconvert --help or ffconvert --help effects");
            }
            return;
        }
    };
    println!("{text}");
}

fn print_help() {
    println!(
        "{}",
        r#"ffconvert - a friendlier FFmpeg wrapper

Usage:
  ffconvert INPUT OUTPUT [OPTIONS]

Options:
  --resolution WIDTHxHEIGHT  Scale the video, e.g. 1920x1080
  --fps FPS                  Set the output frame rate
  --saturation N             Color saturation (1 is normal)
  --contrast N               Contrast (1 is normal)
  --brightness N             Brightness (-1 to 1, 0 is normal)
  --gamma N                  Gamma (1 is normal)
  --hue DEGREES              Rotate hue colors
  --sharpness N              Sharpen (0 is off)
  --blur N                   Blur strength (0 is off)
  --noise N                  Add video noise (0 is off)
  --grayscale N              Grayscale strength (0 to 1)
  --vignette N               Vignette angle in radians
  --exposure N               Exposure adjustment
  --temperature K            Color temperature in Kelvin
  --vibrance N               Color vibrance (1 is normal)
  --sepia N                  Sepia strength (0 to 1)
  --rotate DEGREES           Rotate the video
  --flip horizontal|vertical Flip the video
  --crop WIDTHxHEIGHT        Crop from the center
  --pad WIDTHxHEIGHT        Pad to a centered canvas
  --image-format FORMAT      Frame export format: png, jpg, jpeg, or webp
  --pixelate N               Pixel block size
  --denoise N                Denoise strength
  --deband N                 Debanding strength
  --edge-detect N            Edge detection strength
  --emboss N                 Emboss strength
  --invert true|false        Invert colors
  --mirror true|false        Mirror horizontally
  --deinterlace true|false  Deinterlace video
  --deflicker true|false     Reduce flicker
  --deshake true|false       Stabilize shaky video
  --audio-quality 1-10       10 is normal, 1 is extreme lo-fi (default: 10)
  --audio-copy               Copy the audio stream without filtering/re-encoding
  --force                    Overwrite an existing output without asking
  -h, --help                 Show this help

Paths with spaces or parentheses should be quoted by your shell:
  ffconvert "thing.mp4" "thing (2).mp4" --fps 60 --saturation 1.2 --contrast 1.1"#
    );
}

fn closest_option(input: &str) -> Option<&'static str> {
    const OPTIONS: &[&str] = &[
        "--resolution",
        "--fps",
        "--saturation",
        "--contrast",
        "--brightness",
        "--gamma",
        "--hue",
        "--sharpness",
        "--blur",
        "--noise",
        "--grayscale",
        "--vignette",
        "--exposure",
        "--temperature",
        "--vibrance",
        "--sepia",
        "--rotate",
        "--flip",
        "--crop",
        "--pad",
        "--pixelate",
        "--denoise",
        "--deband",
        "--edge-detect",
        "--emboss",
        "--invert",
        "--mirror",
        "--deinterlace",
        "--deflicker",
        "--deshake",
        "--image-format",
        "--audio-quality",
        "--audio-copy",
        "--force",
        "--help",
    ];
    let input = input.strip_prefix("--").unwrap_or(input);
    OPTIONS
        .iter()
        .copied()
        .map(|option| {
            (
                option,
                edit_distance(input, option.trim_start_matches("--")),
            )
        })
        .filter(|(_, distance)| *distance <= 3 || *distance <= input.len() / 3)
        .min_by_key(|(_, distance)| *distance)
        .map(|(option, _)| option)
}

fn edit_distance(left: &str, right: &str) -> usize {
    let mut previous: Vec<usize> = (0..=right.chars().count()).collect();
    for (i, left_char) in left.chars().enumerate() {
        let mut current = vec![i + 1];
        for (j, right_char) in right.chars().enumerate() {
            let cost = usize::from(left_char != right_char);
            current.push(
                (current[j] + 1)
                    .min(previous[j + 1] + 1)
                    .min(previous[j] + cost),
            );
        }
        previous = current;
    }
    previous[right.chars().count()]
}

fn fail(message: impl AsRef<str>) -> ExitCode {
    eprintln!("ffconvert: {}", message.as_ref());
    ExitCode::from(2)
}

fn parse_args() -> Result<Options, String> {
    let mut positional = Vec::new();
    let mut resolution = None;
    let mut fps = None;
    let mut video_filters = Vec::new();
    let mut image_format = "png".to_string();
    let mut audio_quality = 10;
    let mut audio_copy = false;
    let mut force = false;
    let args: Vec<String> = env::args().skip(1).collect();
    let mut i = 0;

    while i < args.len() {
        match args[i].as_str() {
            "-h" | "--help" => {
                if let Some(topic) = args.get(i + 1) {
                    print_topic_help(topic);
                } else {
                    print_help();
                }
                std::process::exit(0);
            }
            "--force" => force = true,
            "--audio-copy" => audio_copy = true,
            "--image-format" => {
                if i + 1 >= args.len() {
                    return Err("--image-format needs png, jpg, jpeg, or webp".into());
                }
                image_format = args[i + 1].to_ascii_lowercase();
                if !matches!(image_format.as_str(), "png" | "jpg" | "jpeg" | "webp") {
                    return Err("--image-format needs png, jpg, jpeg, or webp".into());
                }
                i += 1;
            }
            "--resolution" | "--fps" | "--audio-quality" => {
                if i + 1 >= args.len() {
                    return Err(format!("{} needs a value", args[i]));
                }
                let value = args[i + 1].clone();
                match args[i].as_str() {
                    "--resolution" => {
                        let parts: Vec<&str> = value.split('x').collect();
                        if parts.len() != 2 || parts.iter().any(|p| p.parse::<u32>().is_err()) {
                            return Err(
                                "resolution must look like WIDTHxHEIGHT, e.g. 1920x1080".into()
                            );
                        }
                        resolution = Some(value);
                    }
                    "--fps" => {
                        if value.parse::<f64>().map_or(true, |v| v <= 0.0) {
                            return Err("fps must be a positive number".into());
                        }
                        fps = Some(value);
                    }
                    "--audio-quality" => {
                        audio_quality = value
                            .parse::<u8>()
                            .map_err(|_| "audio quality must be an integer from 1 to 10")?;
                        if !(1..=10).contains(&audio_quality) {
                            return Err("audio quality must be from 1 to 10".into());
                        }
                    }
                    _ => unreachable!(),
                }
                i += 1;
            }
            "--saturation" | "--contrast" | "--brightness" | "--gamma" | "--hue"
            | "--sharpness" | "--blur" | "--noise" | "--grayscale" | "--vignette"
            | "--exposure" | "--temperature" | "--vibrance" | "--sepia" | "--rotate"
            | "--pixelate" | "--denoise" | "--deband" | "--edge-detect" | "--emboss" => {
                if i + 1 >= args.len() {
                    return Err(format!("{} needs a number", args[i]));
                }
                let value = args[i + 1]
                    .parse::<f64>()
                    .map_err(|_| format!("{} needs a number", args[i]))?;
                if !value.is_finite() {
                    return Err(format!("{} needs a finite number", args[i]));
                }
                let filter = match args[i].as_str() {
                    "--saturation" if value >= 0.0 => format!("eq=saturation={value}"),
                    "--contrast" if value >= 0.0 => format!("eq=contrast={value}"),
                    "--brightness" if (-1.0..=1.0).contains(&value) => {
                        format!("eq=brightness={value}")
                    }
                    "--gamma" if value > 0.0 => format!("eq=gamma={value}"),
                    "--hue" => format!("hue=h={value}"),
                    "--sharpness" => format!("unsharp=5:5:{value}"),
                    "--blur" if value >= 0.0 => format!("boxblur=lr={value}:lp=1"),
                    "--noise" if value >= 0.0 => format!("noise=alls={value}:allf=t+u"),
                    "--grayscale" if (0.0..=1.0).contains(&value) => {
                        format!("hue=s={}", 1.0 - value)
                    }
                    "--vignette" if value >= 0.0 => format!("vignette=angle={value}"),
                    "--exposure" => format!("exposure=exposure={value}"),
                    "--temperature" if (1000.0..=40000.0).contains(&value) => {
                        format!("colortemperature=temperature={value}")
                    }
                    "--vibrance" => format!("vibrance=intensity={}", value - 1.0),
                    "--sepia" if (0.0..=1.0).contains(&value) => {
                        format!(
                            "colorchannelmixer=rr=0.393:rg=0.769:rb=0.189:gr=0.349:gg=0.686:gb=0.168:br=0.272:bg=0.534:bb=0.131:pc={value}"
                        )
                    }
                    "--rotate" => format!("rotate={value}*PI/180"),
                    "--pixelate" if value >= 1.0 => {
                        format!("scale=iw/{value}:ih/{value},scale=iw:ih:flags=neighbor")
                    }
                    "--denoise" if value >= 0.0 => {
                        format!("hqdn3d={value}:{value}:{value}:{value}")
                    }
                    "--deband" if value >= 0.0 => {
                        format!("deband=1thr={value}:2thr={value}:3thr={value}:4thr={value}")
                    }
                    "--edge-detect" if value >= 0.0 => {
                        format!("edgedetect=low={}:high={}", value / 2.0, value)
                    }
                    "--emboss" if value >= 0.0 => {
                        "convolution=0=-1 0 0 0:1=-1 0 0 0:2=-1 0 0 0".to_string()
                    }
                    _ => {
                        return Err(format!("invalid value for {}", args[i]));
                    }
                };
                video_filters.push(filter);
                i += 1;
            }
            "--invert" | "--mirror" | "--deinterlace" | "--deflicker" | "--deshake" => {
                if i + 1 >= args.len() {
                    return Err(format!("{} needs true or false", args[i]));
                }
                let enabled = match args[i + 1].to_ascii_lowercase().as_str() {
                    "true" | "yes" | "on" => true,
                    "false" | "no" | "off" => false,
                    _ => return Err(format!("{} needs true or false", args[i])),
                };
                if enabled {
                    let filter = match args[i].as_str() {
                        "--invert" => "negate",
                        "--mirror" => "hflip",
                        "--deinterlace" => "yadif",
                        "--deflicker" => "deflicker",
                        "--deshake" => "deshake",
                        _ => unreachable!(),
                    };
                    video_filters.push(filter.to_string());
                }
                i += 1;
            }
            "--flip" | "--crop" | "--pad" => {
                if i + 1 >= args.len() {
                    return Err(format!("{} needs a value", args[i]));
                }
                let value = args[i + 1].clone();
                let filter = match args[i].as_str() {
                    "--flip" => match value.to_ascii_lowercase().as_str() {
                        "horizontal" => "hflip".to_string(),
                        "vertical" => "vflip".to_string(),
                        _ => return Err("--flip needs horizontal or vertical".into()),
                    },
                    "--crop" | "--pad" => {
                        let parts: Vec<&str> = value.split('x').collect();
                        if parts.len() != 2 || parts.iter().any(|part| part.parse::<u32>().is_err())
                        {
                            return Err(format!("{} must look like WIDTHxHEIGHT", args[i]));
                        }
                        if args[i] == "--crop" {
                            format!(
                                "crop={}:{}:(iw-{})/2:(ih-{})/2",
                                parts[0], parts[1], parts[0], parts[1]
                            )
                        } else {
                            format!("pad={}:{}:(ow-iw)/2:(oh-ih)/2", parts[0], parts[1])
                        }
                    }
                    _ => unreachable!(),
                };
                video_filters.push(filter);
                i += 1;
            }
            value if value.starts_with('-') => {
                let message = match closest_option(value) {
                    Some(suggestion) => {
                        format!("unknown option: {value}\nDid you mean {suggestion}?")
                    }
                    None => format!("unknown option: {value}"),
                };
                return Err(message);
            }
            value => positional.push(value.to_string()),
        }
        i += 1;
    }

    if positional.len() != 2 {
        return Err("expected INPUT and OUTPUT; use --help for examples".into());
    }
    if positional[0] == positional[1] {
        return Err("input and output must be different files".into());
    }
    if audio_copy && audio_quality != 10 {
        return Err("--audio-copy cannot be combined with --audio-quality below 10".into());
    }
    Ok(Options {
        input: positional[0].clone(),
        output: positional[1].clone(),
        resolution,
        fps,
        video_filters,
        image_format,
        audio_quality,
        audio_copy,
        force,
    })
}

fn available_encoders() -> String {
    Command::new("ffmpeg")
        .args(["-hide_banner", "-encoders"])
        .output()
        .map(|o| {
            String::from_utf8_lossy(&o.stdout).into_owned() + &String::from_utf8_lossy(&o.stderr)
        })
        .unwrap_or_default()
}

fn has_encoder(encoders: &str, name: &str) -> bool {
    encoders
        .lines()
        .any(|line| line.split_whitespace().nth(1) == Some(name))
}

fn nvidia_available() -> bool {
    Command::new("nvidia-smi")
        .arg("-L")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|status| status.success())
}

fn audio_settings(quality: u8) -> (Option<String>, &'static str) {
    match quality {
        10 => (None, "normal"),
        9 => (Some("aresample=48000".into()), "very high"),
        8 => (Some("aresample=48000".into()), "high"),
        7 => (Some("aresample=44100".into()), "medium-high"),
        6 => (
            Some("aresample=44100,acrusher=bits=12:mix=0.15".into()),
            "medium",
        ),
        5 => (
            Some("aresample=32000,acrusher=bits=12:mix=0.35".into()),
            "compressed",
        ),
        4 => (
            Some("aresample=22050,acrusher=bits=10:mix=0.65".into()),
            "lo-fi",
        ),
        3 => (
            Some("aresample=16000,acrusher=bits=8:mix=1".into()),
            "very lo-fi",
        ),
        2 => (
            Some("aresample=11025,acrusher=bits=8:mix=1".into()),
            "strong lo-fi",
        ),
        1 => (
            Some("aresample=8000,acrusher=bits=6:mix=1".into()),
            "extreme lo-fi",
        ),
        _ => unreachable!(),
    }
}

fn search_for_input(input: &str) -> Result<Option<String>, String> {
    let home =
        env::var("HOME").map_err(|_| "could not determine your home directory".to_string())?;
    let file_name = Path::new(input)
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| "input does not contain a valid file name".to_string())?;

    println!("Could not find {input}. Searching {home} and below...");
    let result = Command::new("find")
        .args([home.as_str(), "-type", "f", "-name", file_name, "-print0"])
        .output()
        .map_err(|error| format!("could not search your home directory: {error}"))?;
    let mut matches: Vec<String> = result
        .stdout
        .split(|byte| *byte == 0)
        .filter(|path| !path.is_empty())
        .map(|path| String::from_utf8_lossy(path).into_owned())
        .collect();
    matches.sort();
    matches.dedup();

    for candidate in matches {
        let folder = Path::new(&candidate)
            .parent()
            .and_then(|path| path.to_str())
            .unwrap_or("the folder shown");
        let prompt = format!("Do you mean the file in:\n  {folder}? [Y/N]");
        if ask_yes_no(&prompt)? {
            return Ok(Some(candidate));
        }
    }

    Ok(None)
}

// Keep the confirmation prompt on separate lines in Konsole so resizing
// cannot make the short Y/N response look attached to a wrapped path.
fn ask_yes_no(prompt: &str) -> Result<bool, String> {
    println!("{prompt}");
    io::stdout()
        .flush()
        .map_err(|error| format!("could not show prompt: {error}"))?;

    // Pipes and redirected stdin still use the normal line-based fallback.
    if !io::stdin().is_terminal() {
        let mut answer = String::new();
        io::stdin()
            .read_line(&mut answer)
            .map_err(|error| format!("could not read answer: {error}"))?;
        return match answer.trim().to_ascii_lowercase().as_str() {
            "y" | "yes" => Ok(true),
            "n" | "no" => Ok(false),
            _ => Err("please answer Y or N".into()),
        };
    }

    // Konsole sends arrow keys as escape sequences. Raw mode lets us ignore
    // those bytes and Ctrl+C without letting them corrupt the confirmation.
    let saved = Command::new("stty")
        .arg("-g")
        .output()
        .map_err(|error| format!("could not inspect terminal settings: {error}"))?;
    let saved = String::from_utf8_lossy(&saved.stdout).trim().to_string();
    let raw = Command::new("stty")
        .args(["-icanon", "-echo", "-isig", "min", "1", "time", "0"])
        .status()
        .map_err(|error| format!("could not configure terminal input: {error}"))?;
    if !raw.success() {
        return Err("could not configure terminal input".into());
    }

    let mut stdin = io::stdin();
    let answer = loop {
        let mut byte = [0_u8; 1];
        if let Err(error) = stdin.read_exact(&mut byte) {
            let _ = Command::new("stty").args(saved.split_whitespace()).status();
            return Err(format!("could not read answer: {error}"));
        }
        match byte[0].to_ascii_lowercase() {
            b'y' => break true,
            b'n' => break false,
            // Ignore Enter, Ctrl+C, and arrow-key escape sequences.
            _ => {}
        }
    };

    let restored = Command::new("stty").args(saved.split_whitespace()).status();
    println!("{}", if answer { "Y" } else { "N" });
    if !restored.map_or(false, |status| status.success()) {
        return Err("could not restore terminal settings".into());
    }
    Ok(answer)
}

fn convert_frames(options: &Options, input: &str) -> ExitCode {
    let directory = Path::new(&options.output);
    let existed = directory.exists();
    if existed && !directory.is_dir() {
        return fail("frame output must be a directory");
    }
    if existed && !options.force {
        let prompt = format!(
            "Frame output directory already exists:\n  {}\nOverwrite matching frames? [Y/N]",
            options.output
        );
        match ask_yes_no(&prompt) {
            Ok(true) => {}
            Ok(false) => return fail("not overwriting existing frames"),
            Err(error) => return fail(error),
        }
    }
    if let Err(error) = fs::create_dir_all(directory) {
        return fail(format!("could not create frame directory: {error}"));
    }

    let codec = match options.image_format.as_str() {
        "png" => "png",
        "jpg" | "jpeg" => "mjpeg",
        "webp" => "libwebp",
        _ => return fail("unsupported image format"),
    };
    let pattern = directory.join(format!("%d.{}", options.image_format));
    let mut filters = options.video_filters.clone();
    if let Some(size) = &options.resolution {
        filters.push(format!("scale={size}:force_original_aspect_ratio=decrease"));
    }
    let filter_chain = filters.join(",");
    let mut command = Command::new("ffmpeg");
    command.args([
        "-hide_banner",
        if options.force { "-y" } else { "-n" },
        "-i",
        input,
        "-map",
        "0:v:0",
    ]);
    if !filter_chain.is_empty() {
        command.args(["-vf", &filter_chain]);
    }
    if let Some(rate) = &options.fps {
        command.args(["-r", rate]);
    }
    command.args(["-start_number", "1", "-c:v", codec, "-an"]);
    if codec == "mjpeg" {
        command.args(["-q:v", "2"]);
    } else if codec == "libwebp" {
        command.args(["-quality", "90"]);
    }
    command.arg(&pattern);

    println!("Extracting frames: {} -> {}", input, pattern.display());
    println!("Image format: {}", options.image_format);
    if let Some(rate) = &options.fps {
        println!("FPS: {rate}");
    }
    let status = match command.status() {
        Ok(status) => status,
        Err(error) => return fail(format!("could not start ffmpeg: {error}")),
    };
    if !status.success() {
        return fail(format!("ffmpeg exited with {status}"));
    }

    let count = fs::read_dir(directory)
        .ok()
        .into_iter()
        .flatten()
        .filter_map(Result::ok)
        .filter(|entry| {
            entry.path().extension().and_then(|ext| ext.to_str())
                == Some(options.image_format.as_str())
        })
        .count();
    if count == 0 {
        return fail("FFmpeg reported success but no frames were created");
    }
    println!(
        "Done: {} {} frame(s) in {}",
        count,
        options.image_format,
        directory.display()
    );
    ExitCode::SUCCESS
}

fn main() -> ExitCode {
    let options = match parse_args() {
        Ok(o) => o,
        Err(e) => return fail(e),
    };
    let input = if Path::new(&options.input).is_file() {
        options.input.clone()
    } else {
        match search_for_input(&options.input) {
            Ok(Some(found)) => found,
            Ok(None) => return fail(format!("could not find input file: {}", options.input)),
            Err(error) => return fail(error),
        }
    };
    if options.output.ends_with('/') || Path::new(&options.output).is_dir() {
        return convert_frames(&options, &input);
    }
    let overwrite = if Path::new(&options.output).exists() && !options.force {
        let prompt = format!(
            "Output already exists:\n  {}\nOverwrite it? [Y/N]",
            options.output
        );
        match ask_yes_no(&prompt) {
            Ok(true) => true,
            Ok(false) => return fail("not overwriting existing output"),
            Err(error) => return fail(error),
        }
    } else {
        options.force
    };
    if options.output.ends_with('/') {
        return fail("output must be a file path");
    }

    let output_ext = Path::new(&options.output)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    let encoders = available_encoders();
    let use_nvenc = nvidia_available() && has_encoder(&encoders, "h264_nvenc");
    let (video_codec, audio_codec, video_preset) = match output_ext.as_str() {
        "webm" => {
            let v = if has_encoder(&encoders, "libsvtav1") {
                "libsvtav1"
            } else if has_encoder(&encoders, "libvpx-vp9") {
                "libvpx-vp9"
            } else {
                return fail("FFmpeg has neither libsvtav1 nor libvpx-vp9 for WebM output");
            };
            (v, "libopus", "6")
        }
        "mkv" => {
            let v = if use_nvenc {
                "h264_nvenc"
            } else if has_encoder(&encoders, "libx264") {
                "libx264"
            } else {
                return fail("FFmpeg has no usable H.264 encoder");
            };
            (v, "libopus", "medium")
        }
        _ => {
            let v = if use_nvenc {
                "h264_nvenc"
            } else if has_encoder(&encoders, "libx264") {
                "libx264"
            } else {
                return fail("FFmpeg has no usable H.264 encoder");
            };
            (v, "aac", "medium")
        }
    };

    let mut video_filters = options.video_filters.clone();
    if let Some(size) = &options.resolution {
        video_filters.push(format!("scale={size}:force_original_aspect_ratio=decrease"));
    }
    let video_filter_chain = video_filters.join(",");

    let mut command = Command::new("ffmpeg");
    command.arg("-hide_banner");
    command.arg(if overwrite { "-y" } else { "-n" });
    command.args(["-i", &input, "-map", "0:v:0", "-map", "0:a?"]);
    if !video_filter_chain.is_empty() {
        command.args(["-vf", &video_filter_chain]);
    }
    if let Some(rate) = &options.fps {
        command.args(["-r", rate]);
    }
    command.args(["-c:v", video_codec, "-preset", video_preset]);
    if video_codec == "h264_nvenc" {
        command.args(["-cq", "23"]);
    } else if video_codec == "libsvtav1" {
        command.args(["-crf", "30"]);
    } else if video_codec == "libvpx-vp9" {
        command.args(["-crf", "32", "-b:v", "0"]);
    } else {
        command.args(["-crf", "20"]);
    }

    if options.audio_copy {
        command.args(["-c:a", "copy"]);
    } else {
        command.args(["-c:a", audio_codec]);
        if audio_codec == "aac" {
            command.args(["-b:a", "160k"]);
        } else {
            command.args(["-b:a", "128k"]);
        }
        if let Some(filter) = audio_settings(options.audio_quality).0 {
            command.args(["-af", &filter]);
        }
    }
    command.args(["-map_metadata", "0", &options.output]);

    let (_, quality_name) = audio_settings(options.audio_quality);
    let audio_label = if options.audio_copy {
        "copied".to_string()
    } else {
        format!(
            "{} (quality {}/10, {})",
            audio_codec, options.audio_quality, quality_name
        )
    };
    println!("Converting: {} -> {}", input, options.output);
    println!("Video: {} | audio: {}", video_codec, audio_label);
    if let Some(size) = &options.resolution {
        println!("Resolution: {size}");
    }
    if let Some(rate) = &options.fps {
        println!("FPS: {rate}");
    }

    let status = match command.status() {
        Ok(s) => s,
        Err(e) => return fail(format!("could not start ffmpeg: {e}")),
    };
    if !status.success() {
        return fail(format!("ffmpeg exited with {status}"));
    }
    if !Path::new(&options.output).is_file() {
        return fail("FFmpeg reported success but output file was not created");
    }

    let probe = Command::new("ffprobe")
        .args([
            "-v",
            "error",
            "-show_entries",
            "format=duration",
            "-of",
            "default=noprint_wrappers=1:nokey=1",
            &options.output,
        ])
        .output();
    match probe {
        Ok(p) if p.status.success() => println!(
            "Done: {} ({})",
            options.output,
            String::from_utf8_lossy(&p.stdout).trim()
        ),
        _ => println!("Done: {} (ffprobe could not read duration)", options.output),
    }
    let _ = fs::metadata(&options.output);
    ExitCode::SUCCESS
}
