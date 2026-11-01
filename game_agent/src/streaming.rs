// this is wheere we send the frames to the clients over the interner through signalling server (which is my old laptop)
use anyhow::{Context, Result};
use std::sync::Arc;

use webrtc::{
    api::{
        interceptor_registry::register_default_interceptors, // for lost packets
        media_engine::{MediaEngine, MIME_TYPE_H264},
        APIBuilder,
    },
    ice_transport::ice_server::RTCIceServer,
    interceptor::registry::Registry,
    peer_connection::{configuration::RTCConfiguration, RTCPeerConnection},
    rtp_transceiver::rtp_codec::RTCRtpCodecCapability,
    track::track_local::{track_local_static_sample::TrackLocalStaticSample, TrackLocal},
};
