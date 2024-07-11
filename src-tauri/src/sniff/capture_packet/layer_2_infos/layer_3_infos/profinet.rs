use std::fmt;
use serde::Serialize;

#[repr(u16)]
#[derive(Debug, Serialize, Clone, Eq, PartialEq, Hash)]
pub enum FrameId {
    Unicast = 0xC000,
    Multicast = 0xF800,
    GetReqSetReqGetRespSetResp = 0xFEFD,
    IdentifyReq = 0xFEFE,
    IdentifyResp = 0xFEFF,
}

impl FrameId {
    fn from_u16(value: u16) -> Option<FrameId> {
        match value {
            0xC000..=0xF7FF => Some(FrameId::Unicast),
            0xF800..=0xFBFF => Some(FrameId::Multicast),
            0xFEFD => Some(FrameId::GetReqSetReqGetRespSetResp),
            0xFEFE => Some(FrameId::IdentifyReq),
            0xFEFF => Some(FrameId::IdentifyResp),
            _ => None,
        }
    }
}

impl Default for FrameId {
    fn default() -> Self {
        FrameId::Unicast
    }
}

#[derive(Debug, Serialize, Clone, Eq, PartialEq, Hash)]
pub enum DataStatus {
    Good = 0x80,
    Bad = 0x00,
}

impl Default for DataStatus {
    fn default() -> Self {
        DataStatus::Good
    }
}

#[derive(Debug, Serialize, Clone, Eq, PartialEq, Hash)]
pub enum TransferStatus {
    Running = 0x01,
    Stopped = 0x00,
}

impl Default for TransferStatus {
    fn default() -> Self {
        TransferStatus::Stopped
    }
}

#[derive(Debug, Default, Serialize, Clone, Eq, PartialEq, Hash)]
pub struct ProfinetPacket {
    pub source: String,
    pub destination: String,
    pub frame_id: FrameId,
    pub user_data: Vec<u8>,
    pub cycle_counter: u16,
    pub data_status: DataStatus,
    pub transfer_status: TransferStatus,
}

impl ProfinetPacket {
    pub fn new(data: &[u8]) -> Option<ProfinetPacket> {
        println!("Received data: {:02X?}", data);
        if data.len() < 10 {
            println!("Data too short to be a valid Profinet packet.");
            return None;
        }

        let frame_id_value = u16::from_be_bytes([data[12], data[13]]);
        println!("Frame ID value: {:04x}", frame_id_value);
        let frame_id = FrameId::from_u16(frame_id_value)?;
        println!("Parsed Frame ID: {:?}", frame_id);  // Debug trace
        let cycle_counter = u16::from_be_bytes([data[14], data[15]]);
        println!("Cycle Counter: {}", cycle_counter);
        let data_status = match data[16] {
            0x80 => DataStatus::Good,
            0x00 => DataStatus::Bad,
            _ => {
                println!("Invalid data status: {:02x}", data[16]);
                return None;
            }
        };
        println!("Data Status: {:?}", data_status);
        let transfer_status = match data[17] {
            0x00 => TransferStatus::Stopped,
            0x01 => TransferStatus::Running,
            value => {
                println!("Invalid transfer status value: {:02x}", value);  // Debug trace
                return None;
            }
        };
        println!("Transfer Status: {:?}", transfer_status);  // Debug trace

        let user_data = data[18..].to_vec();
        println!("User Data: {:?}", user_data);  // Debug trace

        Some(ProfinetPacket {
            source: "source_ip_placeholder".to_string(),  // Replace with actual source IP extraction logic
            destination: "destination_ip_placeholder".to_string(),  // Replace with actual destination IP extraction logic
            frame_id,
            user_data,
            cycle_counter,
            data_status,
            transfer_status,
        })
    }
}

impl fmt::Display for ProfinetPacket {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "Source: {}", self.source)?;
        writeln!(f, "Destination: {}", self.destination)?;
        writeln!(f, "Frame ID: {:?}", self.frame_id)?;
        writeln!(f, "User Data: {:?}", self.user_data)?;
        writeln!(f, "Cycle Counter: {}", self.cycle_counter)?;
        writeln!(f, "Data Status: {:?}", self.data_status)?;
        writeln!(f, "Transfer Status: {:?}", self.transfer_status)?;
        Ok(())
    }
}
