use anyhow::{Context, Result, anyhow};
use std::{env, process::Stdio};
use tokio::{
    io::AsyncWriteExt,
    process::Command,
    time::{Duration, timeout},
};

// ffmpeg -f gdigrab -framerate 60 -i desktop -an -vf scale=1280:720 -pix_fmt yuv420p -f rawvideo pipe:1 > NUL : COMMAND FOR POWERSHELL

pub async fn ingest() -> Result<()> {
    let video_input = env::var("GAME_CAPTURE_INPUT").unwrap_or_else(|_| "desktop".to_string());
    let audio_device =
        env::var("GAME_CAPTURE_AUDIO_DEVICE").unwrap_or_else(|_| "default".to_string());
    let output = env::var("GAME_CAPTURE_OUTPUT").unwrap_or_else(|_| "capture.mp4".to_string());

    let width = 1280usize;
    let height = 720usize;

    // YUV420P: 1.5 bytes per pixel.
    let frame_size = width * height * 3 / 2;

    let mut child = Command::new("ffmpeg")
        .args([
            "-vf",
            "scale=1280:720",
            "-an",
            "-c:v",
            "libx264",
            "-preset",
            "ultrafast",
            "-tune",
            "zerolatency",
            "-crf",
            "23",
            "-pix_fmt",
            "yuv420p",
            "-g",
            "60",
            "-bf",
            "0",
            "-f",
            "h264",
            "pipe:1",
            "-hide_banner",
            "-loglevel",
            "warning",
            "-f",
            "gdigrab",
            "-framerate",
            "60",
            "-i",
            &video_input,
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .context("Could not start FFmpeg. ")?;

    let mut stdout = child
    .stdout
    .take()
    .context("Could not access FFmpeg output")?;

    let mut frame = vec![0u8; frame_size];
    let mut frame_number = 0u64;

    println!(
        "Capturing video from {video_input} and audio from WASAPI device {audio_device} to {output}."
    );

    loop {
        match stdout.read_exact(&mut frame).await {
            Ok(_) => {
                frame_number += 1;

                if frame_number % 60 == 0 {
                    println!("Captured {frame_number} frames");
                }

                // One complete YUV420P frame is now in `frame`.
                // Next: feed frames into the video encoder/WebRTC pipeline.
            }
            Err(error)
                if error.kind() == std::io::ErrorKind::UnexpectedEof =>
            {
                break;
            }
            Err(error) => return Err(error.into()),
        }
    }

    
    tokio::select! {
        result = child.wait() => {
            let status = result.context("Failed while waiting for FFmpeg")?;
            if status.success() {
                Ok(())

            } else {
                Err(anyhow!("FFmpeg exited with status {status}"))
            }
        }
        signal = tokio::signal::ctrl_c() => {
            signal.context("Failed to listen for Ctrl+C")?;
            if let Some(mut stdin) = child.stdin.take() {
                stdin.write_all(b"q\n").await.context("Failed to stop FFmpeg gracefully")?;
            }

            match timeout(Duration::from_secs(10), child.wait()).await {
                Ok(Ok(status)) if status.success() => Ok(()),
                Ok(Ok(status)) => Err(anyhow!("FFmpeg exited with status {status}")),
                Ok(Err(error)) => Err(error).context("Failed while waiting for FFmpeg to stop"),
                Err(_) => {
                    child.kill().await.context("Failed to terminate FFmpeg")?;
                    let _ = child.wait().await;
                    Err(anyhow!("FFmpeg did not stop within 10 seconds"))
                }
            }
        }
    }

}
