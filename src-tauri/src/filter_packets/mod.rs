use crate::SonarState;
use log::info;

pub fn update_filter(state: &mut SonarState, enable_ipv6_filter: bool) {
    let mut filter_ipv6_lock = state.filter_ipv6.lock().expect("Failed to lock mutex");
    *filter_ipv6_lock = enable_ipv6_filter;

    info!("IPv6 filter is now {}", enable_ipv6_filter);
}
