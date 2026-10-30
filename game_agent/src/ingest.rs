use anyhow::{Context, Result, anyhow};
use std::{env, process::Stdio};
use tokio::{
    io::AsyncWriteExt,
    process::Command,
    time::{Duration, timeout},
};

pub async fn ingest() -> Result<()> {
    let input = env::var("GAME_CAPTURE_INPUT").unwrap_or_else(|_| "desktop".to_string());
    let output = env::var("GAME_CAPTURE_OUTPUT").unwrap_or_else(|_| "capture.mp4".to_string());

    let mut child = Command::new("ffmpeg")
        .args([
            "-y",
            "-f",
            "gdigrab",
            "-framerate",
            "60",
            "-i",
            &input,
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
            "-movflags",
            "+faststart",
            &output,
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .context("Could not start FFmpeg.")?;

    println!("Capturing {input} to {output}.");

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
