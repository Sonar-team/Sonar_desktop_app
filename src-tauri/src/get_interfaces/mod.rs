use pnet::datalink;
use log::info;

pub fn get_interfaces() -> Vec<String> {
    let interfaces = datalink::interfaces();
    info!("Fetching network interfaces");

    let mut names: Vec<String> = interfaces
        .iter()
        .map(|iface| {
            iface.name.clone()
            //println!("Found interface: {}", name);
        })
        .collect();
    let all = String::from("all");
    names.push(all);

    names
}
