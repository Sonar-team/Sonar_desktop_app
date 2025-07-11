use packet_parser::PacketFlow;

use serde::Serialize;



#[derive(Debug, Clone, Serialize)]
pub struct PacketMinimal<'a> {
    pub ts_sec: i64,
    pub ts_usec: i32,
    pub caplen: u32,
    pub len: u32,
    pub flow: Option<PacketFlow<'a>>, 
}
