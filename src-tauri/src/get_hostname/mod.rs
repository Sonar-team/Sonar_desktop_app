use whoami::fallible;

pub fn hostname_to_s() -> String {
    fallible::hostname().unwrap()
}
