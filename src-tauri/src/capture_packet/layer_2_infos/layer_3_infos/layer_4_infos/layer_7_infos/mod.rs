use pnet::packet::Packet;
use pnet::packet::dhcp::DhcpPacket;

#[derive(Default,Debug)]
pub struct Layer7Infos {
    proto: Option<String>
}

trait PacketPyload {
    fn proto(&self) -> Layer7Infos;
}

pub fn get_layer_7_infos(data: &[u8]) -> Layer7Infos {
    Layer7Infos {
        proto: None,
    }
}