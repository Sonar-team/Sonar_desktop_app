use csv::StringRecord;

use crate::state::flow_matrix::LabelEntry;

const LABEL_CSV_FORMAT_ERROR: &str = "CSV labels invalide : format attendu mac,ip,label";

pub fn parse_label_csv(csv_data: &str) -> Result<Vec<LabelEntry>, String> {
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(false)
        .flexible(true)
        .trim(csv::Trim::All)
        .from_reader(csv_data.as_bytes());

    let mut labels = Vec::new();
    let mut first_data_row = true;

    for record in reader.records() {
        let record = record.map_err(|_| label_csv_format_error())?;

        if record.iter().all(|field| field.trim().is_empty()) {
            continue;
        }

        if first_data_row && is_header(&record) {
            first_data_row = false;
            continue;
        }
        first_data_row = false;

        if record.len() != 3 {
            return Err(label_csv_format_error());
        }

        let mac = clean_label_field(record.get(0).unwrap_or_default()).to_ascii_lowercase();
        let ip = clean_label_field(record.get(1).unwrap_or_default()).to_string();
        let label = clean_label_field(record.get(2).unwrap_or_default()).to_string();

        if label.is_empty() || (mac.is_empty() && ip.is_empty()) {
            return Err(label_csv_format_error());
        }

        labels.push(LabelEntry { mac, ip, label });
    }

    Ok(labels)
}

fn label_csv_format_error() -> String {
    LABEL_CSV_FORMAT_ERROR.to_string()
}

fn clean_label_field(field: &str) -> &str {
    field.trim()
}

fn is_header(record: &StringRecord) -> bool {
    if record.len() != 3 {
        return false;
    }

    record
        .iter()
        .map(|field| field.trim().to_ascii_lowercase())
        .eq(["mac", "ip", "label"].into_iter().map(String::from))
}

#[cfg(test)]
mod tests {
    use super::parse_label_csv;

    #[test]
    fn parses_required_mac_ip_label_format() {
        let labels = parse_label_csv("mac,ip,label\naa:bb:cc:dd:ee:ff,192.168.1.10,PLC\n")
            .unwrap();

        assert_eq!(labels.len(), 1);
        assert_eq!(labels[0].mac, "aa:bb:cc:dd:ee:ff");
        assert_eq!(labels[0].ip, "192.168.1.10");
        assert_eq!(labels[0].label, "PLC");
    }

    #[test]
    fn accepts_ip_only_and_mac_only_rows() {
        let labels = parse_label_csv(
            "mac,ip,label\n,8.8.8.8,DNS\nAA:BB:CC:DD:EE:FF,,automate\n",
        )
        .unwrap();

        assert_eq!(labels.len(), 2);
        assert_eq!(labels[0].mac, "");
        assert_eq!(labels[0].ip, "8.8.8.8");
        assert_eq!(labels[0].label, "DNS");
        assert_eq!(labels[1].mac, "aa:bb:cc:dd:ee:ff");
        assert_eq!(labels[1].ip, "");
        assert_eq!(labels[1].label, "automate");
    }

    #[test]
    fn accepts_headerless_csv() {
        let labels = parse_label_csv("aa:bb:cc:dd:ee:ff,192.168.1.10,PLC\n").unwrap();

        assert_eq!(labels.len(), 1);
        assert_eq!(labels[0].label, "PLC");
    }

    #[test]
    fn rejects_rows_without_three_columns() {
        assert!(parse_label_csv("8.8.8.8,DNS\n").is_err());
    }

    #[test]
    fn rejects_rows_without_label_or_match_key() {
        assert!(parse_label_csv("mac,ip,label\n,,orphan\n").is_err());
        assert!(parse_label_csv("mac,ip,label\n8.8.8.8,,\n").is_err());
    }
}
