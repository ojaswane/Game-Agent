# Game-Agent

This repository is moving toward direct WebRTC game streaming.

## Target media path

```text
Gaming agent ─── WebRTC video/audio over UDP ───► Browser
      │                                               │
      └──── signaling through the control server ────┘
```

The control server should coordinate WebRTC setup messages for a session. It should not carry the video. The agent currently supports worker registration, heartbeats, and `START_SESSION` / `STOP_SESSION` control messages. Direct WebRTC capture, signaling, and browser playback are not implemented yet.

The previous MediaMTX, RTMP publishing, and test-server setup has been removed. The next implementation step is to agree on the signaling messages that the control server can relay between the worker and the browser (offer, answer, and ICE candidates).
