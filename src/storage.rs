use super::metrics::{Digest, Metrics};

use influx_db_client::UdpClient;
use influx_db_client::{Point, Value};

pub trait Storage {
    fn process(&self, metrics: Digest);
}

impl<D> Storage for Option<D>
where D: Storage {
    fn process(&self, metrics: Digest) {
        match self.as_ref() {
            Some(db) => db.process(metrics),
            None => println!("{}", serde_json::to_string_pretty(&metrics).unwrap())
        }
    }
}

const AUDIO_IN: &'static str = "inbound-audio";
const VIDEO_IN: &'static str = "inbound-video";
const AUDIO_OUT: &'static str = "outbound-audio";
const VIDEO_OUT: &'static str = "outbound-video";

const ROOM_TAG: &'static str = "room";
const MEMBER_TAG: &'static str = "member";

impl Storage for UdpClient {
    fn process(&self, metrics: Digest) {
        let room_id = metrics.room_id;
        let member_id = metrics.member_id;

        let audio_in = metrics.inbound.audio.convert(AUDIO_IN, room_id, member_id);
        let audio_out = metrics.outbound.audio.convert(AUDIO_OUT, room_id, member_id);
        let video_in = metrics.inbound.video.convert(VIDEO_IN, room_id, member_id);
        let video_out = metrics.outbound.video.convert(VIDEO_OUT, room_id, member_id);
        self.write_point(audio_in).unwrap();
        self.write_point(audio_out).unwrap();
        self.write_point(video_in).unwrap();
        self.write_point(video_out).unwrap();
    }
}

fn convert_janus_id(id: u64) -> Value {
    //InfluxDB 1.8 supports only i64
    //InfluxDB 2.0 will support u64
    Value::String(id.to_string())
}

fn convert_timestamp(js_timestamp: f64) -> i64 {
    //JavaScript uses f64 for storing timestamps:
    //integral part denotes milliseconds,
    //fractal part denotes microseconds with 5μs precision.
    //InfluxDB uses i64 and stores time in nanoseconds
    ((js_timestamp * 1000.0).round() as i64) * 1000
}

trait ToPoint where Self: std::marker::Sized {
    fn convert(self, measurement: &str, room_id: u64, member_id: u64) -> Point {
        let point = Self::create(measurement, room_id, member_id);
        self.enrich(point)
    }

    fn create(measurement: &str, room_id: u64, member_id: u64) -> Point {
        Point::new(measurement)
            .add_tag(ROOM_TAG, convert_janus_id(room_id))
            .add_tag(MEMBER_TAG, convert_janus_id(member_id))
    }

    fn enrich(self, point: Point) -> Point;
}

impl ToPoint for Metrics {
    fn enrich(self, point: Point) -> Point {
        let mut point = point.add_timestamp(convert_timestamp(self.timestamp));

        if let Some(value) = self.jitter {
            point = point.add_field("jitter",Value::Float(value as f64));
        }
        if let Some(value) = self.round_trip_time {
            point = point.add_field("round-trip-time",Value::Float(value as f64));
        }
        if let Some(value) = self.bytes_sent {
            point = point.add_field("bytes-sent", Value::Integer(value as i64));
        }
        if let Some(value) = self.bytes_received {
            point = point.add_field("bytes-received",Value::Integer(value as i64));
        }
        if let Some(value) = self.packets_sent {
            point = point.add_field("packets-sent",Value::Integer(value as i64));
        }
        if let Some(value) = self.packets_received {
            point = point.add_field("packets-received",Value::Integer(value as i64));
        }
        if let Some(value) = self.packets_lost {
            point = point.add_field("packets-lost",Value::Integer(value as i64));
        }
        if let Some(value) = self.retransmitted_bytes_sent {
            point = point.add_field("retransmitted-bytes-sent", Value::Integer(value as i64));
        }
        if let Some(value) = self.header_bytes_sent {
            point = point.add_field("header-bytes-sent", Value::Integer(value as i64));
        }
        if let Some(value) = self.header_bytes_received {
            point = point.add_field("header-bytes-received", Value::Integer(value as i64));
        }
        if let Some(value) = self.quality {
            point = point.add_field("fps", Value::Integer(value.frames_per_second as i64));
            point = point.add_field("width", Value::Integer(value.frame_resolution.width as i64));
            point = point.add_field("height", Value::Integer(value.frame_resolution.height as i64));
        }

        point
    }
}
