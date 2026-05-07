#![cfg(target_os = "macos")]

embed_plist::embed_info_plist!("../src/Info.plist");

embed_plist::embed_launchd_plist!("../src/launchd.plist");

fn get_info_plist_section() -> &'static [u8] {
    extern "C" {
        #[link_name = "\x01section$start$__TEXT$__info_plist"]
        static START: u8;

        #[link_name = "\x01section$end$__TEXT$__info_plist"]
        static END: u8;
    }
    unsafe {
        let start: *const u8 = &START;
        let end: *const u8 = &END;
        let len = end as usize - start as usize;
        core::slice::from_raw_parts(start, len)
    }
}

fn get_launchd_plist_section() -> &'static [u8] {
    extern "C" {
        #[link_name = "\x01section$start$__TEXT$__launchd_plist"]
        static START: u8;

        #[link_name = "\x01section$end$__TEXT$__launchd_plist"]
        static END: u8;
    }
    unsafe {
        let start: *const u8 = &START;
        let end: *const u8 = &END;
        let len = end as usize - start as usize;
        core::slice::from_raw_parts(start, len)
    }
}

macro_rules! to_str {
    ($bytes:expr) => {
        core::str::from_utf8($bytes).unwrap_or_else(|error| {
            panic!(
                "Invalid bytes at '{}:{}:{}': {}",
                file!(),
                line!(),
                column!(),
                error,
            )
        })
    };
}

#[test]
fn info_plist_contents() {
    let embedded = to_str!(get_info_plist_section());
    let included = include_str!("../src/Info.plist");
    assert_eq!(embedded, included);
}

#[test]
fn launchd_plist_contents() {
    let embedded = to_str!(get_launchd_plist_section());
    let included = include_str!("../src/launchd.plist");
    assert_eq!(embedded, included);
}
