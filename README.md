# Game-Agent

The repository now includes a local streaming prototype: the Windows Rust agent captures the RDR2 window with FFmpeg, encodes H.264 (and optional AAC audio), and publishes it to MediaMTX over RTMP. MediaMTX serves the stream to browsers over WebRTC.

## Start the streaming server and viewer

Install Docker Desktop, then from this repository run:

```powershell
docker compose up -d
```

This starts MediaMTX and a small viewer page. On the server computer, open `http://localhost:8080`. Enter the session ID sent in the server's `START_SESSION` command. The agent publishes session `abc123` at `rtmp://<server-host>:1935/live/abc123`; the viewer plays `http://<server-host>:8889/live/abc123` using MediaMTX's WebRTC player.

For a gaming PC on the same network as the server laptop, set `STREAM_PUBLISH_URL` on the gaming PC to `rtmp://<server-laptop-LAN-IP>:1935/live`. For example, if the laptop's LAN IP is `192.168.1.50`, set it to `rtmp://192.168.1.50:1935/live`. In the viewer, use `http://192.168.1.50:8889` as the MediaMTX address. Set `AUDIO_DEVICE` to an FFmpeg DirectShow device name if game audio is needed; otherwise the stream is silent.

MediaMTX's WebRTC playback needs TCP port 8889 and UDP port 8189 reachable from the browser. Docker Compose maps both ports. On a different computer, edit `streaming/mediamtx.yml` and set `webrtcAdditionalHosts` to the server laptop's LAN IP so ICE sends a reachable address to the browser. Internet playback requires a public address/domain, firewall/NAT configuration, HTTPS for the website, and often STUN/TURN depending on the network.

## Important

The included MediaMTX config is for local development: it allows anyone who can reach the server to publish or view streams. Do not expose it to the public internet as-is. Production needs authentication, TLS, access control, and appropriate network/ICE configuration. The existing Rust control server must still send `START_SESSION` / `STOP_SESSION`; this repo does not contain that control server.
