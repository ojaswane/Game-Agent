# Game-Agent

The Windows agent registers with the Rust control server and publishes an active desktop session to an RTMP ingest using FFmpeg. FFmpeg captures the desktop with `gdigrab`, encodes H.264 with low-latency settings, and optionally captures/encodes audio as AAC.

## Streaming setup

Install FFmpeg on the gaming host and make `ffmpeg.exe` available on `PATH`. Configure `STREAM_PUBLISH_URL` as the streaming host's RTMP base ingest URL, such as `rtmp://stream-host/live`. On `START_SESSION`, the agent publishes to `{STREAM_PUBLISH_URL}/{session_id}`. Configure the streaming host to expose that stream to the browser (for example through WebRTC); the browser should connect to the streaming host, not to the worker-control WebSocket.

Set `AUDIO_DEVICE` to an FFmpeg DirectShow input such as `audio=Stereo Mix` to include audio. Leave it unset for silent video. The device must exist on the gaming host. `SERVER_URL`, `WORKER_ID`, and `MAX_SESSIONS` configure the existing worker connection.

This first capture backend publishes the whole Windows desktop. It does not launch RDR2, implement a browser player, or configure the streaming host. The server and browser need to agree on how the session ID is mapped to the viewer's stream URL.
