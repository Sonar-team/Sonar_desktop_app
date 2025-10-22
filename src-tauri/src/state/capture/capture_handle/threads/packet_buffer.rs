use std::sync::Mutex;

use pcap::PacketHeader;

pub struct PacketBuffer {
    pub header: PacketHeader,
    pub data: Box<[u8]>,
}

impl PacketBuffer {
    pub fn new(buffer_size: usize) -> Self {
        Self {
            header: PacketHeader {
                ts: libc::timeval {
                    tv_sec: 0,
                    tv_usec: 0,
                },
                caplen: 0,
                len: 0,
            },
            data: vec![0u8; buffer_size].into_boxed_slice(),
        }
    }
}
pub struct PacketBufferPool {
    pool: Mutex<Vec<PacketBuffer>>,
}

impl PacketBufferPool {
    pub fn new(pool_size: usize, buffer_size: usize) -> Self {
        let mut pool = Vec::with_capacity(pool_size);
        for _ in 0..pool_size {
            pool.push(PacketBuffer::new(buffer_size));
        }
        Self { pool: Mutex::new(pool) }
    }

    pub fn get(&self) -> Option<PacketBuffer> {
        self.pool.lock().ok()?.pop()
    }

    pub fn put(&self, buffer: PacketBuffer) {
        if let Ok(mut guard) = self.pool.lock() {
            guard.push(buffer);
        }
    }
}

