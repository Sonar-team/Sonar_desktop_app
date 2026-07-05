use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};

use crossbeam::queue::SegQueue;
use pcap::PacketHeader;

/// Taille des buffers « standard » : Ethernet + VLAN + MTU 1500 avec marge.
/// Couvre la quasi-totalité des trames d'un réseau classique ; au-delà
/// (jumbo frames, GRO/TSO), on bascule sur des buffers de taille snaplen.
const SMALL_BUFFER_SIZE: usize = 2048;

/// Buffer de paquet à capacité fixe
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

    /// Copie sûre depuis un PacketHeader + payload brut.
    #[inline]
    pub fn write_from_parts(&mut self, src_header: &PacketHeader, src_payload: &[u8]) {
        // Invariants garantis en amont :
        // - le pool fournit un buffer de capacité >= caplen (voir `get`)
        // - src_payload.len() == caplen (libpcap)
        self.header = *src_header;

        let caplen = self.header.caplen as usize;

        // Copie "pile la taille"
        self.data[..caplen].copy_from_slice(&src_payload[..caplen]);
    }

    #[inline]
    pub fn as_slice(&self) -> &[u8] {
        let n = self.header.caplen as usize;
        &self.data[..n]
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        self.data.len()
    }
}

impl AsRef<[u8]> for PacketBuffer {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        self.as_slice()
    }
}

/// Compteurs d'activité du pool, pour le monitoring et le benchmark
/// (`capture_timing`).
#[derive(Debug, Default, Clone, Copy)]
pub struct PoolStats {
    pub small_allocated: usize,
    pub large_allocated: usize,
    pub exhausted: u64,
}

/// Pool lock-free de PacketBuffer à deux classes de tailles.
///
/// Les buffers ne sont pas préalloués : ils sont créés à la demande, dans la
/// limite de `max_buffers` au total (toutes classes confondues), puis recyclés
/// via les files lock-free. La mémoire suit donc la profondeur réelle du canal
/// (haute-eau) au lieu du pire cas `max_buffers × snaplen`.
pub struct PacketBufferPool {
    small: SegQueue<PacketBuffer>,
    large: SegQueue<PacketBuffer>,
    small_size: usize,
    large_size: usize,
    /// Nombre d'allocations restantes autorisées (budget global).
    remaining: AtomicUsize,
    small_allocated: AtomicUsize,
    large_allocated: AtomicUsize,
    /// Nombre de `get` refusés faute de budget (paquets perdus côté app).
    exhausted: AtomicU64,
}

impl PacketBufferPool {
    pub fn new(pool_size: usize, buffer_size: usize) -> Self {
        Self {
            small: SegQueue::new(),
            large: SegQueue::new(),
            small_size: SMALL_BUFFER_SIZE.min(buffer_size),
            large_size: buffer_size,
            remaining: AtomicUsize::new(pool_size),
            small_allocated: AtomicUsize::new(0),
            large_allocated: AtomicUsize::new(0),
            exhausted: AtomicU64::new(0),
        }
    }

    /// Récupère un buffer d'au moins `caplen` octets (ou None si budget épuisé).
    #[inline]
    pub fn get(&self, caplen: usize) -> Option<PacketBuffer> {
        if caplen <= self.small_size {
            if let Some(buffer) = self.small.pop() {
                return Some(buffer);
            }
            if self.try_reserve() {
                self.small_allocated.fetch_add(1, Ordering::Relaxed);
                return Some(PacketBuffer::new(self.small_size));
            }
            // Budget épuisé : un jumbo au repos peut encore servir.
        }

        if caplen <= self.large_size {
            if let Some(buffer) = self.large.pop() {
                return Some(buffer);
            }
            if self.try_reserve() {
                self.large_allocated.fetch_add(1, Ordering::Relaxed);
                return Some(PacketBuffer::new(self.large_size));
            }
        }

        self.exhausted.fetch_add(1, Ordering::Relaxed);
        None
    }

    /// Remet un buffer dans la file correspondant à sa classe de taille.
    #[inline]
    pub fn put(&self, buffer: PacketBuffer) {
        if buffer.capacity() <= self.small_size {
            self.small.push(buffer);
        } else {
            self.large.push(buffer);
        }
    }

    /// Réserve une unité du budget d'allocation global (CAS sans verrou).
    #[inline]
    fn try_reserve(&self) -> bool {
        self.remaining
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |remaining| {
                remaining.checked_sub(1)
            })
            .is_ok()
    }

    pub fn stats(&self) -> PoolStats {
        PoolStats {
            small_allocated: self.small_allocated.load(Ordering::Relaxed),
            large_allocated: self.large_allocated.load(Ordering::Relaxed),
            exhausted: self.exhausted.load(Ordering::Relaxed),
        }
    }

    /// Mémoire actuellement allouée par les buffers du pool, en octets.
    pub fn allocated_bytes(&self) -> usize {
        let stats = self.stats();
        stats.small_allocated * self.small_size + stats.large_allocated * self.large_size
    }

    /// Optionnel : debug seulement (O(n))
    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.small.len() + self.large.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn header(caplen: u32) -> PacketHeader {
        PacketHeader {
            ts: libc::timeval {
                tv_sec: 0,
                tv_usec: 0,
            },
            caplen,
            len: caplen,
        }
    }

    #[test]
    fn small_packet_gets_small_buffer() {
        let pool = PacketBufferPool::new(4, 65536);
        let buffer = pool.get(1500).expect("buffer standard");
        assert_eq!(buffer.capacity(), SMALL_BUFFER_SIZE);
    }

    #[test]
    fn jumbo_packet_gets_large_buffer() {
        let pool = PacketBufferPool::new(4, 65536);
        let buffer = pool.get(9000).expect("buffer jumbo");
        assert_eq!(buffer.capacity(), 65536);
    }

    #[test]
    fn buffers_are_recycled_per_class() {
        let pool = PacketBufferPool::new(4, 65536);

        let small = pool.get(100).unwrap();
        let large = pool.get(4000).unwrap();
        pool.put(small);
        pool.put(large);

        // Recyclage : aucune nouvelle allocation
        let _ = pool.get(100).unwrap();
        let _ = pool.get(4000).unwrap();
        let stats = pool.stats();
        assert_eq!(stats.small_allocated, 1);
        assert_eq!(stats.large_allocated, 1);
    }

    #[test]
    fn global_budget_is_enforced() {
        let pool = PacketBufferPool::new(2, 65536);

        let a = pool.get(100).expect("1er buffer");
        let _b = pool.get(9000).expect("2e buffer");
        assert!(pool.get(100).is_none(), "budget épuisé");
        assert_eq!(pool.stats().exhausted, 1);

        // Après restitution, plus de refus.
        pool.put(a);
        assert!(pool.get(100).is_some());
    }

    #[test]
    fn snaplen_smaller_than_small_class_uses_single_tier() {
        let pool = PacketBufferPool::new(2, 512);
        let buffer = pool.get(512).expect("buffer");
        assert_eq!(buffer.capacity(), 512);
        assert!(pool.get(513).is_none(), "au-delà du snaplen");
    }

    #[test]
    fn write_from_parts_roundtrip() {
        let pool = PacketBufferPool::new(2, 65536);
        let mut buffer = pool.get(4).unwrap();
        let payload = [1u8, 2, 3, 4];

        buffer.write_from_parts(&header(4), &payload);

        assert_eq!(buffer.as_slice(), &payload);
        assert_eq!(buffer.as_ref().len(), 4);
    }

    #[test]
    fn allocated_bytes_tracks_lazy_growth() {
        let pool = PacketBufferPool::new(1000, 65536);
        assert_eq!(pool.allocated_bytes(), 0, "aucune préallocation");

        let buffer = pool.get(1500).unwrap();
        assert_eq!(pool.allocated_bytes(), SMALL_BUFFER_SIZE);
        pool.put(buffer);
        assert_eq!(pool.allocated_bytes(), SMALL_BUFFER_SIZE, "recyclé, pas libéré");
    }
}
