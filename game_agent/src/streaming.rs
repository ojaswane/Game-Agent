use std::{process::Stdio, time::Duration};

use anyhow::{bail, Context, Result};
use tokio::{process::{Child, Command}, time::timeout};

use crate::config::AgentConfig;

/// Owns one FFmpeg publisher. Dropping the child also stops the local capture process.
pub struct StreamProcess {
    child: Child,
    pub publish_url: String,
}

impl StreamProcess {
    pub async fn start(config: &AgentConfig, session_id: &str) -> Result<Self> {
        if !cfg!(windows) {
            bail!("The current capture backend uses Windows DirectShow and gdigrab");
        }

        let base = config.stream_publish_url.trim_end_matches('/');
        if base.is_empty() {
            bail!("STREAM_PUBLISH_URL must point to the streaming host's RTMP ingest");
        }

        let publish_url = format!("{base}/{session_id}");

        let mut command = Command::new("ffmpeg");
        // For testing purpose , we are using RDR2
        command.args([
            "-hide_banner", "-loglevel", "warning", "-f", "gdigrab",
            "-framerate", "30", "-draw_mouse", "1", "-i", "title=Red Dead Redemption 2",
        ]);
        if let Some(device) = &config.audio_device {
            command.args(["-f", "dshow", "-i", device]);
        }
        command.args([
            "-c:v", "libx264", "-preset", "veryfast", "-tune", "zerolatency",
            "-pix_fmt", "yuv420p", "-r", "30", "-g", "60",
        ]);
        if config.audio_device.is_some() {
            command.args(["-c:a", "aac", "-b:a", "128k", "-ar", "48000"]);
        } else {
            command.arg("-an");
        }
        command.args(["-f", "flv", &publish_url]);
        command.stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::inherit());
        command.kill_on_drop(true);

        let mut child = command.spawn().context(
            "could not start FFmpeg; install it and ensure ffmpeg.exe is on PATH",
        )?;
        // Catch immediate startup failures (missing devices, invalid ingest URL, etc.).
        if let Ok(Ok(status)) = timeout(Duration::from_millis(250), child.wait()).await {
            bail!("FFmpeg exited during startup with {status}");
        }

        Ok(Self { child, publish_url })
    }

    pub async fn stop(mut self) -> Result<()> {
        self.child.kill().await.context("failed to stop FFmpeg")?;
        let _ = self.child.wait().await;
        Ok(())
    }
}