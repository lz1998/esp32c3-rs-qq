use nrs_qq::engine::{CoreCryptoRng, TimerProvider};
use smol::net::AsyncToSocketAddrs;
use std::{net::ToSocketAddrs, time::Instant};
// engine provider
pub struct MyTimeProvider {
    pub net_time: i64,
    pub net_begin: Instant,
}
#[inline]
fn read_buf(buf: &[u8]) -> i64 {
    (buf[0] as i64)
        | ((buf[1] as i64) << 8)
        | (buf[2] as i64) << 16
        | (buf[3] as i64) << 24
        | (buf[4] as i64) << 32
        | (buf[5] as i64) << 40
        | (buf[6] as i64) << 48
        | (buf[7] as i64) << 56
}
impl MyTimeProvider {
    pub fn new_form_net_sync<T: ToSocketAddrs>(addr: T) -> Self {
        let buf = &mut [0u8; 8];
        {
            let stream = std::net::TcpStream::connect(addr).unwrap();
            stream.peek(buf).unwrap();
        }
        let net_begin = Instant::now();
        let net_time = read_buf(buf);
        Self {
            net_begin,
            net_time,
        }
    }
    pub async fn new_from_net_async<T: AsyncToSocketAddrs>(addr: T) -> Self {
        let buf = &mut [0u8; 8];
        {
            let stream = smol::net::TcpStream::connect(addr).await.unwrap();
            stream.peek(buf).await.unwrap();
        }
        let net_begin = Instant::now();
        let net_time = read_buf(buf);
        Self {
            net_begin,
            net_time,
        }
    }
    pub fn new() -> Self {
        Self {
            net_begin: Instant::now(),
            net_time: 0,
        }
    }
}
impl TimerProvider for MyTimeProvider {
    fn now_timestamp_nanos(&self) -> i64 {
        let dur = Instant::now() - self.net_begin;
        return self.net_time + dur.as_nanos() as i64;
    }
}

// pub struct MyRandomProvidr;
// impl rand::RngCore for MyRandomProvidr {
//     fn next_u32(&mut self) -> u32 {
//         rand::thread_rng().next_u32()
//     }

//     fn next_u64(&mut self) -> u64 {
//         rand::thread_rng().next_u64()
//     }

//     fn fill_bytes(&mut self, dest: &mut [u8]) {
//         rand::thread_rng().fill_bytes(dest)
//     }

//     fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), rand::Error> {
//         rand::thread_rng().try_fill_bytes(dest)
//     }
// }
// impl rand::CryptoRng for MyRandomProvidr {

// }
// impl CoreCryptoRng for MyRandomProvidr {

// }
