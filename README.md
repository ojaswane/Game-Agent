# Game-Agent

This repository is moving toward direct WebRTC game streaming.

## Target media path

```text
Gaming agent ─── WebRTC video/audio over UDP ───► Browser
      │                                               │
      └──── signaling through the control server ────┘
```

The control server should coordinate WebRTC setup messages for a session. It should not carry the video. The agent currently supports worker registration, heartbeats, and `START_SESSION` / `STOP_SESSION` control messages. Direct WebRTC capture, signaling, and browser playback are not implemented yet.

## Worker registration and heartbeat protocol

The agent registers over HTTP with `POST /workers/register` using this JSON body:

```json
{"id":"gaming-pc-01","max_sessions":1}
```

After registration, every 10 seconds it sends an HTTP `POST` to `/workers/{worker_id}/heartbeat` with this JSON body:

```json
{"active_sessions":0}
```

The worker ID is part of the URL, so it is not repeated in the body. The current count is `0` until WebRTC sessions are implemented. Confirm the request body and any authentication requirements against the control server's API.

The previous MediaMTX, RTMP publishing, and test-server setup has been removed. The next implementation step is to agree on the signaling messages that the control server can relay between the worker and the browser (offer, answer, and ICE candidates).

The agent does not currently open a WebSocket at startup. Its session WebSocket routes require an assigned session ID, and this repository does not include a server API or persistent connection that tells the agent when a session is assigned. Session assignment and WebRTC signaling therefore require a matching server-side protocol before they can be handled by the agent.

## Local Windows capture prototype

Install FFmpeg with the `gdigrab` and `wasapi` inputs, and `libx264` and AAC encoders available on `PATH`, then run:

```powershell
cargo run -- --capture
```

This captures the desktop at 60 fps and the default Windows playback device's loopback audio into `capture.mp4` in the current directory. The audio is the whole output-device mix, not isolated to the game. Press Ctrl+C to stop and finalize the MP4. To capture a window by its title, choose a WASAPI device, and/or change the output path:

```powershell
$env:GAME_CAPTURE_INPUT = "title=Red Dead Redemption 2"
$env:GAME_CAPTURE_AUDIO_DEVICE = "default"
$env:GAME_CAPTURE_OUTPUT = "D:\captures\game.mp4"
cargo run -- --capture
```

Set `GAME_CAPTURE_AUDIO_DEVICE` to a WASAPI playback-device name to capture that device instead of the default. Use FFmpeg's WASAPI device listing to find available names. This is a local file-capture prototype only; it does not yet send video over WebRTC.

## Windows GNU Rust toolchain

The Cargo configuration pins the MSYS2 UCRT64 compiler and archiver for build scripts and the matching `dlltool` for Windows import libraries, while leaving Rust's default linker selection intact. The VS Code workspace prepends `D:\MSYS2\ucrt64\bin` to integrated-terminal `PATH`. Close existing integrated terminals and open a new one after changing the workspace settings.
