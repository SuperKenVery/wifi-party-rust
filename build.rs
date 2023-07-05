use std::io::Result;
use prost_build;


fn main() -> Result<()> {
    prost_build::compile_protos(&["src/partyprotocol/items.proto"], &["src/"])?;
    // panic!("Build script ran!!");
    Ok(())
}