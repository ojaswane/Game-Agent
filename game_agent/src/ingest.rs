// Capture the game screen → encode each frame as video → send the encoded frames over a WebRTC connection.
use std::process::{Command, Stdio};

pub fn ingest() {
    // captured and encoded game data into data packets
    let mut child = command::new("ffmpeg")
        .args([
            "-y", // Overwrite output file
            "-f",
            "gdigrab", // Screen capture device (Windows)
            "-framerate",
            "60", // Target 60 frames per second
            "-video_size",
            "1920x1080", // Capture resolution
            "-i",
            "Red Dead Redemption 2", // Capture the entire desktop (or game window)
            "-c:v",
            "libx264", // Encode video using H.264
            "-preset",
            "ultrafast", // Low CPU overhead for real-time capture
            "-crf",
            "23",                        // Quality setting
            "Red Dead Redemption 2.exe", // Output file (for testing purpose )
        ])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    println!("FFmpeg capture started.");
}
