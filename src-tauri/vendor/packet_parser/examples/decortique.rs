use packet_parser::{Application, DataLink, Internet, Transport};

fn parse_csv_hex_bytes(s: &str) -> Vec<u8> {
    s.split(|c| c == ',' || c == '\n' || c == '\r' || c == '\t' || c == ' ')
        .filter(|t| !t.is_empty())
        .map(|t| u8::from_str_radix(t, 16).expect("octet hex invalide"))
        .collect()
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let bytes_str = r#"64, 6E, E0, EA, FA, 83, FE, AA, 81, 8E, C8, 64, 86, DD, 65, 00, 00, 00, 04, D8, 06, 3D, 2A, 00, 14, 50, 40, 07, 08, 0A, 00, 00, 00, 00, 00, 00, 20, 01, 2A, 04, CE, C0, 11, A3, A3, 97, 18, 42, F3, 91, 8F, 5D, EC, 7D, 01, BB, E8, 00, 7D, C6, E5, A6, F8, 51, BD, 12, 80, 18, 00, 7A, 62, 84, 00, 00, 01, 01, 08, 0A, BD, 92, 95, C7, F7, F1, 38, 51, ED, D8, 97, CA, B0, 25, 1B, 0F, CD, 0C, E6, 97, 6E, 5F, FC, 59, 65, 64, FA, 9D, 19, 86, AD, 4D, CD, 59, E5, 48, 1C, 0F, FA, 35, 75, 90, 17, 5E, 99, 2D, A0, A3, EC, 8D, 32, 40, 3B, 4E, BB, 23, B1, 81, E8, 91, 6F, 5A, A5, 18, EE, F4, E1, 26, EF, E3, 1B, 84, 7F, 56, 86, 88, 67, CF, 26, B0, AC, D9, 26, 80, 83, 3C, EE, FA, 8F, B7, FE, 1C, 3F, 1D, 96, D1, 69, 3B, 67, 7B, 26, D7, 6A, CC, 7F, F4, E0, E9, FF, 9F, 5A, 6B, 51, 76, 68, 91, 00, 89, 1D, E5, 59, 6E, D1, 5C, 93, EC, 2D, 87, 57, 0B, 13, C7, 3C, 95, 88, 15, 62, DC, EB, AD, 7A, CA, CF, 6D, EE, 4F, 88, 72, AB, 2E, 07, DA, CC, 00, AB, F8, 53, 4F, 84, 65, B3, F7, 0A, 93, 62, DD, 46, 6B, ED, 09, 7D, D3, 94, 3C, 49, E6, 02, 54, C2, D1, D1, 1E, 8A, 43, DB, 7B, 2B, B2, 0F, AC, 75, BD, 2D, 12, E6, 1A, 13, 5F, C0, 8F, B8, 17, CF, 27, 79, 36, 30, 52, D5, B8, 69, 87, 12, A0, 68, 15, 10, 51, 3B, CD, 0D, 30, 95, AF, 28, C6, 3A, E2, 43, 00, 6D, 44, D7, 92, FA, A2, 1D, 5A, 86, 6C, 88, E8, 94, 80, 74, E1, BF, 99, 69, D6, BF, 96, 5A, 79, 63, 46, 55, 3D, 7B, 64, 38, 4C, CF, 6A, 8A, C5, 20, 3A, A1, 82, 0E, D3, F4, 6A, 3B, 65, 6C, 5B, B4, 67, 0C, 62, 40, DA, 14, F8, 2C, 8E, 27, CB, 1B, E6, 04, 39, C9, AA, 0F, 9F, 58, F7, 16, 19, 4D, EA, F5, CA, 10, BB, A3, F7, 1D, 3B, EB, 73, D8, 78, E0, 76, 8F, 8A, C2, 0E, 7D, 1D, 98, 4B, DC, DC, D4, 4B, F8, 61, AE, 99, C1, 2A, 73, 07, FC, 4E, DE, 84, 55, 80, BB, 97, 90, 3D, A6, B6, 40, 40, 3B, DF, D3, 17, B6, 5B, 97, D2, 79, D8, B9, CA, 5D, F8, 81, B3, 05, CF, 0E, BE, 82, D1, AA, 4F, D3, 2F, B9, 46, 36, 53, D1, 1E, DE, 23, 27, DB, BF, 82, 45, 38, 70, 01, 7C, 4B, 6F, 69, DA, EC, A4, 16, BB, E5, A1, 13, 8C, 62, E0, DA, 69, DD, 85, 68, B0, 17, B1, C6, B9, DE, F2, AD, 9F, 5A, E0, 4B, AB, 9A, DD, 00, DD, C7, 90, EB, DA, 97, 0A, 5C, 80, D4, 43, 34, C1, A0, A0, 3C, E1, 42, 8E, FA, 2D, 6C, 26, 0C, F7, 8E, 64, 42, 31, 3F, D5, EA, CD, F5, 78, 57, 2E, F6, AB, 4D, F6, D6, B6, D9, B8, 89, B2, BE, 67, F0, C8, AE, 5D, 87, 92, 3C, E8, 9D, F5, 93, 86, B8, EC, B2, 9A, EB, 1E, 6C, 5C, 5B, E4, 65, E3, AD, 4B, 62, C1, 67, 44, 30, 68, A2, 68, FF, 60, 67, BE, 0F, 63, 7F, 5E, 99, 94, 63, 5C, 09, D7, 3C, 2B, C5, C5, BC, 76, F8, FB, 2B, 1F, 00, 41, 7E, E6, 77, 92, CE, A3, 4A, B0, 54, 51, 46, 8D, E9, 15, 24, DB, FE, 12, 74, 63, 82, 4E, 1D, 3F, CC, 03, FD, A2, FF, 01, AE, 8D, 21, C2, 42, B9, 96, BC, 9B, 13, 8A, 0C, E2, 11, 16, 6A, F4, 0B, 21, A3, 2B, 0B, 20, 2A, A8, 58, 74, 30, F0, 3A, 46, E9, FE, 87, F5, 99, 1C, 13, 2C, C9, C0, 9C, E3, 67, 57, 88, 8D, 91, 3D, A2, 82, 61, E0, 7E, 53, 7D, 66, F8, C7, 6A, BB, D0, CF, D6, 02, 36, C8, 80, DF, E4, 9A, D4, 8A, 8B, A9, DF, EF, E0, EF, ED, E8, B0, 04, C7, FC, 86, B9, 14, FC, 4F, 4D, D0, 67, F8, DE, E3, C8, EC, D8, 9A, 47, EA, FD, 43, 85, 23, D8, FF, D9, FD, 1F, A5, 79, 7C, D4, 46, FE, 01, 9F, 8B, 5C, D4, CF, 0B, B6, E6, 80, 0F, 1C, 06, F0, 4D, AF, BC, 00, 9B, 55, 8A, BE, B5, 82, 1C, D5, C5, A6, F9, B2, 4F, E4, 76, 06, CA, 09, 82, 90, 84, 5B, A5, AC, 42, AA, 99, 48, 44, 55, 3F, 75, 22, EF, A0, 8F, 99, B7, E6, 2A, 85, 8C, DD, 1D, 73, 76, B5, 52, FA, 2D, DD, 87, D4, F8, 94, 52, 92, A3, 16, 54, F4, 03, 2A, 9E, 6D, C8, 65, 84, BC, 88, 2B, FD, 06, 3E, 43, 9F, B7, 01, DA, 03, 8B, 23, 79, 1A, 07, 06, A1, 67, 2B, D6, D7, 02, 34, CE, EF, 53, 40, C9, 75, A4, 73, F8, F5, 24, 74, 36, 72, A2, 84, E2, 20, 98, D5, 25, B6, FF, 48, C5, 4C, 0D, 79, FE, 2D, 67, EA, 4B, 56, 19, 53, 6E, F1, 82, FE, 18, 1D, EF, 5C, 64, 09, 61, 13, 8E, D1, E7, BB, B7, 95, 47, 52, 95, CA, 34, 18, B8, AB, 5B, 59, 43, 07, F7, 33, 8E, 56, 89, B2, FE, A6, AE, D8, 3A, 08, C3, 56, F4, E4, D0, 72, DA, D9, B5, B3, E3, 8B, D9, A4, C5, A6, 32, C5, F0, 24, E8, 92, E8, 53, 41, DA, 28, 5E, B2, 09, 8A, 7D, 1D, 11, 4B, A8, 66, 2E, 6F, 5C, 33, 51, 3C, C0, D5, D0, D0, 18, 6A, E7, AA, DA, B3, 33, 4D, 03, A8, 64, 4C, 37, 74, A1, 6B, D9, 85, CC, 19, 8F, 48, 01, 2B, BE, 5D, 9C, 95, 24, 72, 93, 6E, 7B, 06, C9, E6, 63, DD, B0, CD, C0, FD, BC, F0, 7E, 19, D1, 10, 64, FE, 5F, 9E, 6F, 81, D7, 44, 09, 81, 33, 1F, 2F, AA, B3, F6, 94, 66, AF, 1C, D7, D8, A2, 8C, 99, F6, 80, ED, 88, A2, 4E, 27, E5, 3A, E2, B6, D2, 32, 3A, A7, 59, 2A, 0D, 16, 90, 94, EA, F5, 13, 4D, 42, 1F, 66, 93, 4A, 21, E7, 5A, 6D, 65, 32, CA, A0, C2, C8, 66, 97, BA, 0B, 4C, 3C, C4, 84, 08, 1E, F8, C9, 4F, 26, 09, A8, B6, 48, 52, 7A, E6, 92, 6D, 72, EE, CB, A7, 18, F5, 1E, 61, CE, 40, 5F, 36, C2, 5E, 20, 97, 8E, 40, D5, D9, DC, 76, DE, C6, 06, E7, 3D, 20, 56, C1, 5A, 69, FB, E1, 69, 63, A0, 9E, 1A, C0, A4, FC, BF, 92, 2D, 74, 7D, 8F, 29, E7, 08, F2, 41, F5, 65, B5, A1, 88, 32, A6, 5F, F7, E4, 1A, 7E, C7, EC, 8B, 90, 3D, 7C, E0, 5C, F2, 98, BE, AC, 64, 1D, 1C, 94, D8, F8, EE, B7, C3, 62, 2B, 84, A5, 0D, FB, 8D, F3, DB, 8D, 12, 1E, BD, A1, 38, 38, 10, 4F, 12, 91, 50, D8, E8, F0, 78, 04, 29, 5D, 30, E5, 9E, 18, 4C, 4F, 4B, 00, 7E, 3E, 62, 42, 0A, 4F, C8, E2, 93, 14, 4F, 38, F8, 28, DE, 4F, F7, 4C, 88, 85, 89, 25, 2D, 1D, E1, 1B, C0, 17, FC, 77, 2A, 18, 32, 40, F6, 82"#;
    println!("Bytes str: {}", bytes_str);
    let frame = parse_csv_hex_bytes(bytes_str);
    println!("Frame: {}, {:?}", frame.len(), frame);

    let data_link = DataLink::try_from(frame.as_slice())?;
    println!("Data link: {:?}", data_link);
    let mut internet = match Internet::try_from(data_link.payload) {
        Ok(internet) => Some(internet),
        Err(e) => return Err(e.into()),
    };
    println!("Internet: {:?}", internet);
    let transport = match internet.as_mut() {
        Some(internet) => match Transport::try_from(internet.payload) {
            Ok(transport) => Some(transport),
            Err(e) => return Err(e.into()),
        },
        None => None,
    };
    println!("Transport: {:?}", transport);

    let application = match &transport {
        Some(t) => match t.payload {
            Some(p) => Application::try_from(p).ok(),
            None => None,
        },
        None => None,
    };
    println!("Application: {:?}", application);

    Ok(())
}
