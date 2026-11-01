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


pub Struct webrtc{
    pub peer: Arc<RTCPeerConnection>,
    pub track : Arc<TrackLocalStaticSample>,
}


pub async fn create_connection() -> Result<(webrtc , String)> {
    // Media engine is for H264 codec dependency
    let mut media_engine = MediaEngine::default();
    media_engine.register_default_codecs()?;

    // to create the registry for the lost packets
    let mut registry = Registry::new();
    register_default_interceptors(&mut registry, &mut media_engine)?;

    let api = APIBuilder::new()
    .with_media_engine(media_engine)
    .with_interceptor_registry(registry)
    .build();

    
}