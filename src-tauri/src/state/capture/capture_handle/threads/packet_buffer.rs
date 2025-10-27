use crossbeam::queue::SegQueue;
use pcap::PacketHeader;

/// Buffer de paquet à capacité fixe (snaplen)
pub struct PacketBuffer {
    pub header: PacketHeader,
    data: Box<[u8]>,
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

    /// Réinitialise le header (utile quand tu remets le buffer dans le pool,
    /// juste pour éviter de garder des valeurs trompeuses).
    #[inline]
    pub fn clear(&mut self) {
        self.header.ts = libc::timeval {
            tv_sec: 0,
            tv_usec: 0,
        };
        self.header.caplen = 0;
        self.header.len = 0;
        // Pas besoin d'effacer data: elle sera réécrite
    }

    /// Copie sûre depuis un PacketHeader + payload brut.

    #[inline]
    pub fn write_from_parts(&mut self, src_header: &PacketHeader, src_payload: &[u8]) {
        // Invariants garantis en amont :
        // - caplen <= snaplen (config capture)
        // - buffer_size == snaplen (pool)
        // - src_payload.len() == caplen (libpcap)
        self.header = *src_header;

        let caplen = self.header.caplen as usize;

        // Garde-fous sans coût en release :
        debug_assert!(
            caplen <= self.data.len(),
            "caplen > buffer_size (invariant brisé)"
        );
        debug_assert!(
            caplen <= src_payload.len(),
            "caplen > src_payload.len() (invariant brisé)"
        );

        // Copie "pile la taille"
        self.data[..caplen].copy_from_slice(&src_payload[..caplen]);
    }

    #[inline]
    pub fn as_slice(&self) -> &[u8] {
        let n = self.header.caplen as usize;
        &self.data[..n]
    }
}

impl AsRef<[u8]> for PacketBuffer {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        self.as_slice()
    }
}

/// Pool lock-free de PacketBuffer
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

    /// Remet un buffer dans le pool (optionnel: clear() ici)
    #[inline]
    pub fn put(&self, mut buffer: PacketBuffer) {
        buffer.clear();
        self.pool.push(buffer);
    }

    /// Optionnel : debug seulement (O(n))
    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.pool.len()
    }
}
