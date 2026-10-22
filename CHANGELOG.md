# Changelog

All notable changes to Simpler-FFmpeg are documented here.

## [Unreleased]

- Added `--autoplay` to open converted videos with `ffplay` after FFmpeg finishes.
- Added video effects for color, lighting, blur, noise, sharpening, grayscale, hue, rotation, pixelation, denoise, deband, edge detection, and more.
- Added boolean video controls using `true` and `false`, including `--invert`, `--mirror`, `--deinterlace`, `--deflicker`, and `--deshake`.
- Added crop, pad, flip, and image-format options.
- Added frame extraction to numbered image files such as `1.png`, `2.png`, and `10.png`.
- Added PNG, JPG, JPEG, and WebP frame output support.
- Added recursive input-file searching under the user's home directory, followed by a root-filesystem (`/`) search if needed, with confirmation prompts.
- Added overwrite confirmation for existing video outputs and frame directories.
- Added detailed feature help with commands such as `ffconvert --help --saturation`.
- Added typo suggestions for misspelled options and help topics.
- Added Konsole-friendly Y/N prompts that ignore arrow-key escape sequences and Ctrl+C input noise.

## [0.1.0]

- Initial Rust-based FFmpeg wrapper.
- Automatic hardware encoder selection with NVIDIA NVENC, Intel Quick Sync, or VAAPI when available, with libx264 fallback.
- Resolution, FPS, audio quality, audio copying, and safe overwrite handling.
