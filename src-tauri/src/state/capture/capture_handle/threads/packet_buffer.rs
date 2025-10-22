use parking_lot::Mutex;
use pcap::PacketHeader;
use std::collections::VecDeque;

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

    pub fn as_slice(&self) -> &[u8] {
        &self.data[..self.header.caplen as usize]
    }
}

pub struct PacketBufferPool {
    pool: Mutex<VecDeque<PacketBuffer>>,
}

impl PacketBufferPool {
    pub fn new(pool_size: usize, buffer_size: usize) -> Self {
        let mut pool = VecDeque::with_capacity(pool_size);
        for _ in 0..pool_size {
            pool.push_back(PacketBuffer::new(buffer_size));
        }
        Self { pool: Mutex::new(pool) }
    }

    pub fn get(&self) -> Option<PacketBuffer> {
        let mut pool = self.pool.lock();
        pool.pop_front()
    }

    pub fn put(&self, buffer: PacketBuffer) {
        let mut pool = self.pool.lock();
        pool.push_back(buffer);
    }
}
