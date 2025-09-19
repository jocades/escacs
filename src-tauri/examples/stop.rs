use std::{sync::Arc, time::Duration};

use anyhow::Result;
use escacs_lib::chess::{self, search, BestMove, Engine, Go, Info, Search, Visitor};
use tokio::sync::{mpsc, oneshot, Notify};
use tracing::info;

#[derive(Default)]
struct GameVisitor {
    info_count: usize,
}

impl chess::Visitor for GameVisitor {
    fn info(&mut self, _: Info) {
        self.info_count += 1;
        info!("info_count = {}", self.info_count);
    }

    fn best(&mut self, best: BestMove) {
        info!(?best);
    }
}

enum Op {
    Go(Go),
}

async fn controller(
    mut engine: Engine,
    mut rx: mpsc::Receiver<Op>,
    mut stop_rx: mpsc::Receiver<oneshot::Sender<()>>,
) -> anyhow::Result<()> {
    let mut visitor = GameVisitor::default();
    engine.uci().await?;
    engine.isready().await?;

    while let Some(op) = rx.recv().await {
        match op {
            Op::Go(job) => {
                info!("new job");
                let cmd = job.to_cmd();
                engine.tx.send(cmd).await?;
                loop {
                    tokio::select! {
                        Some(line) = engine.rx.recv() => match search(&line)? {
                            Some(Search::Info(i)) => visitor.info(i),
                            Some(Search::BestMove(b)) => visitor.best(b),
                            None => continue,
                        },
                        Some(ack) = stop_rx.recv() => {
                            info!("[controller] stop");
                            engine.stop().await?;
                            info!("send ack");
                            _ = ack.send(());
                            break;
                        }
                        else => break,
                    }
                }
                // let fut = engine.go_with(job, &mut visitor);
                // tokio::select! {
                //     res = fut => res?,
                //     Some(ack) = stop_rx.recv() => {
                //         info!("[controller] stop");
                //         engine.stop().await?;
                //         info!("send ack");
                //         _ = ack.send(());
                //     }
                // }
            }
        }
    }
    Ok(())
}

async fn go(job: Go, tx: &mpsc::Sender<Op>) -> anyhow::Result<()> {
    info!("send job");
    tx.send(Op::Go(job)).await?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    setup_logging();

    let (stop_tx, stop_rx) = mpsc::channel(32);
    let (tx, rx) = mpsc::channel(100);
    let engine = Engine::new("stockfish")?;
    tokio::spawn(async move {
        if let Err(e) = controller(engine, rx, stop_rx).await {
            eprintln!("handler error: {e}");
        }
    });

    let job = Go::new().moves(&["d2d4", "g8f6"]).depth(26);

    go(job.clone(), &tx).await?;
    tokio::time::sleep(Duration::from_secs(1)).await;

    let (ack, syn) = oneshot::channel();
    stop_tx.send(ack).await?;
    syn.await?;
    info!("syn_1");

    go(job.clone(), &tx).await?;
    tokio::time::sleep(Duration::from_secs(1)).await;

    let (ack, syn) = oneshot::channel();
    stop_tx.send(ack).await?;
    syn.await?;
    info!("syn_2");

    go(job, &tx).await?;
    tokio::time::sleep(Duration::from_secs(1)).await;

    let (ack, syn) = oneshot::channel();
    stop_tx.send(ack).await?;
    syn.await?;
    info!("syn_3");

    // _ = tokio::signal::ctrl_c().await;

    Ok(())
}

fn setup_logging() {
    use tracing::level_filters::LevelFilter;

    tracing_subscriber::fmt()
        .with_max_level(LevelFilter::TRACE)
        .without_time()
        .with_target(false)
        .compact()
        .init();
}
