macro_rules! include_protos {
    ($package:literal,$version:literal) => {
        include! {
            concat!(
                env!("OUT_DIR"),
                "/",
                $package,
                ".",
                $version,
                ".rs"
            )
        }
    };
}

pub mod priv_msgs_v1 {
    include_protos!("priv_msgs", "v1");
}

#[cfg(test)]
mod test {
    use super::priv_msgs_v1::NewMessage;
    #[test]
    fn is_included() {
        let msg = NewMessage::default();
        println!("{msg:?}");
    }
}
