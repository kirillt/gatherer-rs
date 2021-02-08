use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Digest {
    pub room_id: u64,
    pub member_id: u64,
    pub outbound: Bundle,
    pub inbound: Bundle
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Bundle {
    pub audio: Metrics,
    pub video: Metrics
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Metrics {
    pub jitter: Option<f64>,
    pub round_trip_time: Option<f64>,
    pub bytes_sent: Option<usize>,
    pub bytes_received: Option<usize>,
    pub packets_sent: Option<usize>,
    pub packets_received: Option<usize>,
    pub packets_lost: Option<usize>,
    pub header_bytes_sent: Option<usize>,
    pub header_bytes_received: Option<usize>,
    pub retransmitted_bytes_sent: Option<usize>,
    pub timestamp: f64,

    #[serde(flatten)]
    pub quality: Option<Quality>
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Quality {
    pub frames_per_second: u8,
    pub frame_resolution: Resolution
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Resolution {
    pub height: u16,
    pub width: u16
}
