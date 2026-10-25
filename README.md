# Game-Agent

This repository is moving toward direct WebRTC game streaming.

## Target media path

```text
Gaming agent ─── WebRTC video/audio over UDP ───► Browser
      │                                               │
      └──── signaling through the control server ────┘
```

The control server should coordinate WebRTC setup messages for a session. It should not carry the video. The agent currently supports worker registration, heartbeats, and `START_SESSION` / `STOP_SESSION` control messages. Direct WebRTC capture, signaling, and browser playback are not implemented yet.

## Worker control and heartbeat protocol

The agent keeps its WebSocket connection to `/ws/worker` for registration and incoming session commands. Every 10 seconds it sends an HTTP `POST` to `/workers/{worker_id}/heartbeat` with this JSON body:

```json
{"active_sessions":0}
```

The worker ID is part of the URL, so it is not repeated in the body. The current count is `0` until WebRTC sessions are implemented. Confirm the request body and any authentication requirements against the control server's API.

The previous MediaMTX, RTMP publishing, and test-server setup has been removed. The next implementation step is to agree on the signaling messages that the control server can relay between the worker and the browser (offer, answer, and ICE candidates).
