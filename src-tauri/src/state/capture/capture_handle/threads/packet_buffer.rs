use crossbeam::queue::SegQueue;
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

    #[inline]
    pub fn as_slice(&self) -> &[u8] {
        let n = (self.header.caplen as usize).min(self.data.len());
        &self.data[..n]
    }
}

pub struct PacketBufferPool {
    pool: SegQueue<PacketBuffer>,
}

impl PacketBufferPool {
    pub fn new(pool_size: usize, buffer_size: usize) -> Self {
        let pool = SegQueue::new();
        for _ in 0..pool_size {
            pool.push(PacketBuffer::new(buffer_size));
        }
        Self { pool }
    }

    /// Récupère un buffer disponible (ou None si vide)
    #[inline]
    pub fn get(&self) -> Option<PacketBuffer> {
        self.pool.pop()
    }

    /// Remet un buffer dans le pool
    #[inline]
    pub fn put(&self, buffer: PacketBuffer) {
        self.pool.push(buffer);
    }

    /// Optionnel : pour debug uniquement (coûteux)
    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.pool.len()
    }
}
