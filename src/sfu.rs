use byteorder::{LittleEndian, ReadBytesExt, BigEndian};
use std::os::unix::net::UnixStream;
use std::net::Shutdown;
use std::io::Read;

#[allow(dead_code)]
pub struct LocalSource {
    unix_stream: UnixStream,
    buffer: Vec<u8>,
}

#[allow(dead_code)]
impl LocalSource {
    pub fn connect(unix_path: &str) -> std::io::Result<Self> {
        println!("Connecting to local socket: {}", unix_path);

        let unix_stream = UnixStream::connect(unix_path)?;
        unix_stream.set_nonblocking(true)?;

        Ok(LocalSource {
            unix_stream, buffer: vec![]
        })
    }

    pub fn process_one_message(&mut self) {
        let mut chunk = vec![0; SERVER_MESSAGE_SIZE];
        if let Ok(n) = self.unix_stream.read(&mut chunk) {
            let k = self.buffer.len();
            assert!(k <= SERVER_MESSAGE_SIZE);
            let need = SERVER_MESSAGE_SIZE - k;

            if n < need {
                chunk.into_iter().for_each(|x| self.buffer.push(x));
            } else {
                let next = chunk.split_off(need);
                chunk.into_iter().for_each(|x| self.buffer.push(x));
                process_server_message(&mut self.buffer).unwrap();
                self.buffer = next;
            }
        }
    }

    pub fn shutdown(&self) -> std::io::Result<()> {
        self.unix_stream.shutdown(Shutdown::Both)
    }
}

#[allow(dead_code)]
const SERVER_MESSAGE_SIZE: usize = 25;

#[allow(dead_code)]
fn process_server_message(buffer: &mut Vec<u8>) -> std::io::Result<()> {
    let typ = buffer[0];
    let room = (&buffer[1..9]).read_u64::<LittleEndian>()?;
    let user = (&buffer[9..17]).read_u64::<LittleEndian>()?;
    let seq_num = (&buffer[17..19]).read_u16::<BigEndian>()?;
    let timestamp = (&buffer[19..23]).read_u32::<BigEndian>()?;
    let len = (&buffer[23..25]).read_u16::<LittleEndian>()?;

    Ok(println!("Metrics from SFU: type {}, room {}, user {}\n#{}, timestamp: {}, size: {}",
                typ, room, user, seq_num, timestamp, len))
}