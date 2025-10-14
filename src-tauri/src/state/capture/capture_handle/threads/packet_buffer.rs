use pcap::PacketHeader;
use std::sync::{Arc, Mutex};

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
    pool: Mutex<Vec<Arc<Mutex<PacketBuffer>>>>,
}

impl PacketBufferPool {
    pub fn new(pool_size: usize, buffer_size: usize) -> Self {
        let mut pool = Vec::with_capacity(pool_size);
        for _ in 0..pool_size {
            pool.push(Arc::new(Mutex::new(PacketBuffer::new(buffer_size))));
        }
        Self {
            pool: Mutex::new(pool),
        }
    }

    pub fn get(&self) -> Option<Arc<Mutex<PacketBuffer>>> {
        self.pool.lock().unwrap().pop()
    }

    pub fn put(&self, buffer: Arc<Mutex<PacketBuffer>>) {
        self.pool.lock().unwrap().push(buffer);
    }
}
