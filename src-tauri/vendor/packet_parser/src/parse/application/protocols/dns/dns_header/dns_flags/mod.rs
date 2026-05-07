use crate::errors::application::dns::DnsFlagsError;

/// Verifies the consistency of DNS packet flags.
///
/// DNS packet flags are used to control the behavior of DNS queries and responses.
/// The flags are represented by a 16-bit field in the DNS header, where each bit or group of bits
/// has a specific meaning. Here's a breakdown of the flags:
///
/// - QR (1 bit): Query/Response. 0 for a query, 1 for a response.
/// - Opcode (4 bits): Specifies the type of query. Valid values are 0 to 5.
///   - 0: Standard query (QUERY)
///   - 1: Inverse query (IQUERY)
///   - 2: Server status request (STATUS)
///   - 3-15: Reserved for future use
/// - AA (1 bit): Authoritative Answer. 1 if the server is authoritative for the domain name in the query.
/// - TC (1 bit): Truncated. 1 if the message was truncated due to length greater than that permitted on the transmission channel.
/// - RD (1 bit): Recursion Desired. 1 if the client desires recursive service.
/// - RA (1 bit): Recursion Available. 1 if the server supports recursive queries.
/// - Z (3 bits): Reserved for future use. Must be 0 in all queries and responses.
/// - RCode (4 bits): Response code. Specifies the status of the response. Valid values are 0 to 5.
///   - 0: No error
///   - 1: Format error
///   - 2: Server failure
///   - 3: Name error (only for authoritative name servers)
///   - 4: Not implemented
///   - 5: Refused
///   - 6-15: Reserved for future use
///
/// # Arguments
///
/// * `flags` - A u16 representing the `Flags` field of a DNS packet.
///
/// # Returns
///
/// * `Result<u16, String>` - Ok(flags) if the flags are consistent, Err(message) otherwise.
pub fn verify_dns_flags(flags: u16) -> Result<u16, DnsFlagsError> {
    let (qr, opcode, aa, tc, _rd, ra, z, rcode) = extract_dns_flags(flags);

    verify_z_field(z)?;
    verify_opcode(opcode)?;
    verify_rcode(rcode)?;
    verify_ra_in_query(qr, ra)?;

    if qr == 1 {
        verify_response_flags(opcode, aa, tc, rcode)?;
    }

    Ok(flags)
}

/// Extracts DNS flags into their respective components.
///
/// # Arguments
///
/// * `flags` - A u16 representing the `Flags` field of a DNS packet.
///
/// # Returns
///
/// * `(u16, u16, u16, u16, u16, u16, u16, u16)` - The extracted flags.
fn extract_dns_flags(flags: u16) -> (u16, u16, u16, u16, u16, u16, u16, u16) {
    let qr = (flags >> 15) & 0b1;
    let opcode = (flags >> 11) & 0b1111;
    let aa = (flags >> 10) & 0b1;
    let tc = (flags >> 9) & 0b1;
    let rd = (flags >> 8) & 0b1;
    let ra = (flags >> 7) & 0b1;
    let z = (flags >> 4) & 0b111;
    let rcode = flags & 0b1111;
    // println!(
    //     "qr: {}, opcode: {}, aa: {}, tc: {}, rd: {}, ra: {}, z: {}, rcode: {}",
    //     qr, opcode, aa, tc, rd, ra, z, rcode
    // );
    (qr, opcode, aa, tc, rd, ra, z, rcode)
}

/// Verifies the Z field.
///
/// The Z field is reserved for future use and must always be 0 in both queries and responses.
/// If this field is not 0, it indicates an invalid DNS packet.
///
/// # Arguments
///
/// * `z` - The Z field.
///
/// # Returns
///
/// * `Result<(), String>` - Ok(()) if the Z field is valid, Err(message) otherwise.
#[allow(dead_code)]
fn verify_z_field(z: u16) -> Result<(), DnsFlagsError> {
    if z != 0 {
        return Err(DnsFlagsError::InvalidZField(z));
    }
    Ok(())
}

/// Verifies the opcode field.
///
/// The opcode specifies the type of DNS query. Valid values range from 0 to 5.
/// Values outside this range are reserved and indicate an invalid DNS packet.
///
/// # Arguments
///
/// * `opcode` - The opcode field.
///
/// # Returns
///
/// * `Result<(), String>` - Ok(()) if the opcode is valid, Err(message) otherwise.
fn verify_opcode(opcode: u16) -> Result<(), DnsFlagsError> {
    if opcode > 5 {
        return Err(DnsFlagsError::InvalidOpcode(opcode));
    }
    Ok(())
}

/// Verifies the rcode field.
///
/// The rcode specifies the status of the DNS response. Valid values range from 0 to 5.
/// Values outside this range are reserved and indicate an invalid DNS response.
///
/// # Arguments
///
/// * `rcode` - The rcode field.
///
/// # Returns
///
/// * `Result<(), String>` - Ok(()) if the rcode is valid, Err(message) otherwise.
fn verify_rcode(rcode: u16) -> Result<(), DnsFlagsError> {
    if rcode > 5 {
        return Err(DnsFlagsError::InvalidRCode(rcode));
    }
    Ok(())
}

/// Verifies the RA field in queries.
///
/// The RA (Recursion Available) field should be 0 in queries as it is only set in responses.
/// If RA is set in a query, it indicates an invalid DNS packet.
///
/// # Arguments
///
/// * `qr` - The QR field.
/// * `ra` - The RA field.
///
/// # Returns
///
/// * `Result<(), String>` - Ok(()) if the RA field is valid in queries, Err(message) otherwise.
fn verify_ra_in_query(qr: u16, ra: u16) -> Result<(), DnsFlagsError> {
    if qr == 0 && ra != 0 {
        return Err(DnsFlagsError::RaInQuery(ra));
    }
    Ok(())
}

/// Verifies response flags.
///
/// In DNS responses, certain combinations of flags are not allowed:
/// - In STATUS responses (opcode 2), AA and TC must be 0.
/// - In Server failure responses (rcode 2), AA must be 0.
/// - In Name Error responses (rcode 3), AA must be 1.
/// - In Refused responses (rcode 5), AA must be 0.
///
/// # Arguments
///
/// * `opcode` - The opcode field.
/// * `aa` - The AA field.
/// * `tc` - The TC field.
/// * `rcode` - The rcode field.
///
/// # Returns
///
/// * `Result<(), String>` - Ok(()) if the response flags are valid, Err(message) otherwise.
fn verify_response_flags(opcode: u16, aa: u16, tc: u16, rcode: u16) -> Result<(), DnsFlagsError> {
    if opcode == 2 && (aa != 0 || tc != 0) {
        return Err(DnsFlagsError::AaTcInStatusResponse(aa, tc));
    }

    if rcode == 2 && aa != 0 {
        return Err(DnsFlagsError::AaInServerFailure(aa));
    }

    if rcode == 3 && aa != 1 {
        return Err(DnsFlagsError::AaInNameError(aa));
    }

    if rcode == 5 && aa != 0 {
        return Err(DnsFlagsError::AaInRefused(aa));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verify_z_field() {
        assert_eq!(verify_z_field(0), Ok(()));
        assert_eq!(verify_z_field(1), Err(DnsFlagsError::InvalidZField(1)));
    }

    #[test]
    fn test_verify_opcode() {
        assert_eq!(verify_opcode(0), Ok(()));
        assert_eq!(verify_opcode(5), Ok(()));
        assert_eq!(verify_opcode(6), Err(DnsFlagsError::InvalidOpcode(6)));
    }

    #[test]
    fn test_verify_rcode() {
        assert_eq!(verify_rcode(0), Ok(()));
        assert_eq!(verify_rcode(5), Ok(()));
        assert_eq!(verify_rcode(6), Err(DnsFlagsError::InvalidRCode(6)));
    }
    // Ajoutez d'autres tests similaires pour les autres fonctions de vérification
}
