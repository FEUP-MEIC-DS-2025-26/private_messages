use std::io::Result;

fn main() -> Result<()> {
    let protos = ["proto/priv_msgs/v1.proto"];
    prost_build::compile_protos(&protos, &["proto/"])?;
    Ok(())
}
