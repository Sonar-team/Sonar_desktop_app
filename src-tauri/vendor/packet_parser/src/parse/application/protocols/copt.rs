use std::fmt;

/// COTP PDU Types
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u8)]
pub enum CotpPduType {
    Data = 0xF0,
    ConnectionRequest = 0xE0,
    ConnectionConfirm = 0xD0,
    DisconnectRequest = 0x80,
    DisconnectConfirm = 0xC0,
    TpduError = 0x70,
    Other(u8),
}

impl From<u8> for CotpPduType {
    fn from(value: u8) -> Self {
        match value {
            0xF0 => CotpPduType::Data,
            0xE0 => CotpPduType::ConnectionRequest,
            0xD0 => CotpPduType::ConnectionConfirm,
            0x80 => CotpPduType::DisconnectRequest,
            0xC0 => CotpPduType::DisconnectConfirm,
            0x70 => CotpPduType::TpduError,
            x => CotpPduType::Other(x),
        }
    }
}

#[derive(Debug, Clone)]
pub struct CotpHeader {
    pub length: u8,
    pub pdu_type: CotpPduType,
    pub dst_ref: u16,
    pub src_ref: u16,
    pub class: u8,
    pub extended_formats: bool,
    pub no_explicit_flow_control: bool,
    pub parameters: Vec<CotpParameter>,
}

#[derive(Debug, Clone)]
pub enum CotpParameter {
    TpduSize(u8),       // 0xC0: TPDU size
    SrcTsap(u16),       // 0xC1: Source TSAP
    DstTsap(u16),       // 0xC2: Destination TSAP
    TpduNumber(u8),     // 0xC0 in DT TPDU
    Eot(bool),          // 0x80 in DT TPDU
    Other(u8, Vec<u8>), // Other parameters
}

impl CotpHeader {
    /// Minimum size of a COTP header (3 bytes for basic header)
    pub const MIN_SIZE: usize = 7; // 1 + 1 + 2 + 2 + 1 (for CR/CC)

    /// Parse a COTP header from a byte slice
    pub fn from_bytes(data: &[u8]) -> Result<(Self, usize), &'static str> {
        if data.len() < 3 {
            return Err("Packet too short for COTP header");
        }

        let length = data[0];
        let pdu_type = CotpPduType::from(data[1]);

        if data.len() < length as usize + 1 {
            return Err("Packet shorter than indicated by COTP length");
        }

        let mut offset = 2; // Skip length and PDU type
        let (dst_ref, src_ref, class, extended_formats, no_explicit_flow_control) = match pdu_type {
            CotpPduType::ConnectionRequest
            | CotpPduType::ConnectionConfirm
            | CotpPduType::DisconnectRequest
            | CotpPduType::DisconnectConfirm
            | CotpPduType::TpduError => {
                if data.len() < offset + 6 {
                    return Err("Packet too short for COTP connection header");
                }
                let dst_ref = u16::from_be_bytes([data[offset], data[offset + 1]]);
                let src_ref = u16::from_be_bytes([data[offset + 2], data[offset + 3]]);
                let class = data[offset + 4] >> 4;
                let extended_formats = (data[offset + 4] & 0x04) != 0;
                let no_explicit_flow_control = (data[offset + 4] & 0x01) != 0;
                offset += 5;
                (
                    dst_ref,
                    src_ref,
                    class,
                    extended_formats,
                    no_explicit_flow_control,
                )
            }
            _ => (0, 0, 0, false, false),
        };

        // Parse parameters
        let mut parameters = Vec::new();
        while offset < length as usize + 1 {
            if offset + 1 >= data.len() {
                break;
            }

            let param_type = data[offset];
            let param_len = data[offset + 1] as usize;

            if offset + 2 + param_len > data.len() {
                break;
            }

            let param_data = &data[offset + 2..offset + 2 + param_len];

            let param = match param_type {
                0xC0 => {
                    // TPDU size or TPDU number
                    if pdu_type == CotpPduType::Data {
                        CotpParameter::TpduNumber(param_data[0])
                    } else if param_len == 1 {
                        CotpParameter::TpduSize(param_data[0])
                    } else {
                        CotpParameter::Other(param_type, param_data.to_vec())
                    }
                }
                0xC1 if param_len == 2 => {
                    // Source TSAP
                    CotpParameter::SrcTsap(u16::from_be_bytes([param_data[0], param_data[1]]))
                }
                0xC2 if param_len == 2 => {
                    // Destination TSAP
                    CotpParameter::DstTsap(u16::from_be_bytes([param_data[0], param_data[1]]))
                }
                0x80 if pdu_type == CotpPduType::Data && param_len == 0 => {
                    // EOT
                    CotpParameter::Eot(true)
                }
                _ => CotpParameter::Other(param_type, param_data.to_vec()),
            };

            parameters.push(param);
            offset += 2 + param_len;
        }

        Ok((
            Self {
                length,
                pdu_type,
                dst_ref,
                src_ref,
                class,
                extended_formats,
                no_explicit_flow_control,
                parameters,
            },
            offset,
        ))
    }
}

impl fmt::Display for CotpHeader {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let type_str = match self.pdu_type {
            CotpPduType::Data => "Data (DT)",
            CotpPduType::ConnectionRequest => "Connection Request (CR)",
            CotpPduType::ConnectionConfirm => "Connection Confirm (CC)",
            CotpPduType::DisconnectRequest => "Disconnect Request (DR)",
            CotpPduType::DisconnectConfirm => "Disconnect Confirm (DC)",
            CotpPduType::TpduError => "TPDU Error (ER)",
            CotpPduType::Other(code) => return write!(f, "Unknown PDU Type: 0x{code:02X}"),
        };

        writeln!(f, "COTP: {type_str}")?;
        writeln!(f, "  Length: {}", self.length)?;
        writeln!(f, "  Destination reference: 0x{:04X}", self.dst_ref)?;
        writeln!(f, "  Source reference: 0x{:04X}", self.src_ref)?;

        if self.class != 0 {
            writeln!(f, "  Class: {}", self.class)?;
        }
        if self.extended_formats {
            writeln!(f, "  Extended formats: True")?;
        }
        if self.no_explicit_flow_control {
            writeln!(f, "  No explicit flow control: True")?;
        }

        for param in &self.parameters {
            match param {
                CotpParameter::TpduSize(size) => {
                    let tpdu_size = match size {
                        0x09 => 512,
                        0x0A => 1024,
                        0x0B => 2048,
                        0x0C => 4096,
                        0x0D => 8192,
                        _ => 1 << (*size as u16 + 6),
                    };
                    writeln!(f, "  TPDU size: {tpdu_size} bytes")?;
                }
                CotpParameter::SrcTsap(tsap) => {
                    writeln!(f, "  Source TSAP: 0x{tsap:04X}")?;
                }
                CotpParameter::DstTsap(tsap) => {
                    writeln!(f, "  Destination TSAP: 0x{tsap:04X}")?;
                }
                CotpParameter::TpduNumber(num) => {
                    writeln!(f, "  TPDU Number: {num}")?;
                }
                CotpParameter::Eot(_) => {
                    writeln!(f, "  End of TSDU: Yes")?;
                }
                CotpParameter::Other(code, data) => {
                    write!(f, "  Parameter 0x{code:02X}: ")?;
                    for byte in data {
                        write!(f, "{byte:02X} ")?;
                    }
                    writeln!(f)?;
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_cc_packet() {
        // Example COTP Connection Confirm (CC) packet
        let data = [
            0x11, // Length
            0xD0, // CC (Connection Confirm)
            0x00, 0x01, // Destination reference
            0x00, 0x03, // Source reference
            0x00, // Class and options
            0xC0, 0x01, 0x09, // TPDU size = 512
            0xC1, 0x02, 0x01, 0x00, // Source TSAP = 0x0100
            0xC2, 0x02, 0x01, 0x02, // Destination TSAP = 0x0102
        ];

        let (header, bytes_parsed) = CotpHeader::from_bytes(&data).unwrap();
        assert_eq!(bytes_parsed, data.len());
        assert!(matches!(header.pdu_type, CotpPduType::ConnectionConfirm));
        assert_eq!(header.dst_ref, 0x0001);
        assert_eq!(header.src_ref, 0x0003);

        // Check parameters
        let mut has_tpdu_size = false;
        let mut has_src_tsap = false;
        let mut has_dst_tsap = false;

        for param in &header.parameters {
            match param {
                CotpParameter::TpduSize(0x09) => has_tpdu_size = true,
                CotpParameter::SrcTsap(0x0100) => has_src_tsap = true,
                CotpParameter::DstTsap(0x0102) => has_dst_tsap = true,
                _ => {}
            }
        }

        assert!(has_tpdu_size);
        assert!(has_src_tsap);
        assert!(has_dst_tsap);
    }
}
