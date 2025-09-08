use std::process::{Command, Stdio};

use escacs_lib::chess::{Engine, Go};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut child = Command::new("lc0")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .status()?;
    println!("status: {child:?}");
    // let stdin = child.stdin.take().unwrap();
    // let stdout = child.stdout.take().unwrap();

    let mut engine = Engine::new("lc0")?;
    // let opts = [("Threads", "8"), ("UCI_ShowWDL", "true"), ("MultiPV", "1")];
    // engine.uci().await?;
    // // engine.opts(&opts).await?;
    // engine.isready().await?;
    //
    // let (info, best) = Go::new().depth(15).execute(&mut engine).await?;

    Ok(())
}
