use anyhow::Result;
use escacs_lib::Handle;

#[tokio::main]
async fn main() -> Result<()> {
    let handle = Handle::new();
    Ok(())
}
