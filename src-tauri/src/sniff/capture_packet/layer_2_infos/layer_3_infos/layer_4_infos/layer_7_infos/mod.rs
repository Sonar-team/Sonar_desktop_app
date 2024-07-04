use parse_layer7::parse_layer_7_infos;

pub fn get_layer7_infos(data: &[u8]) -> Option<String> {
    //println!("data: {:?}", data);
    if let Some(infos) = parse_layer_7_infos(data) {
        //println!("infos: {:?}", infos);
        Some(infos.layer_7_protocol)
    } else {
        None
    }
}