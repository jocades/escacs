use std::{fmt::Write, path::Path, process::Stdio, str::FromStr};

use anyhow::{bail, ensure, Context, Result};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    process::{Child, ChildStdin, ChildStdout, Command},
    sync::mpsc,
};
use tracing::{debug, error, trace};

async fn writer(mut stdin: ChildStdin, mut rx: mpsc::Receiver<String>) -> Result<()> {
    while let Some(mut cmd) = rx.recv().await {
        trace!("-> {cmd}");
        cmd.push('\n');
        stdin.write_all(cmd.as_bytes()).await?;
        stdin.flush().await?;
    }
    Ok(())
}

async fn reader(stdout: ChildStdout, tx: mpsc::Sender<String>) -> Result<()> {
    let mut reader = BufReader::new(stdout).lines();
    while let Some(line) = reader.next_line().await? {
        trace!("<- {line}");
        tx.send(line).await?;
    }
    Ok(())
}

pub struct Engine {
    pub id: usize,
    child: Child,
    pub tx: mpsc::Sender<String>,
    pub rx: mpsc::Receiver<String>,
    pub is_searching: bool,
}

impl Engine {
    pub fn new(path: impl AsRef<Path>) -> Result<Self> {
        let mut child = Command::new(path.as_ref())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()?;

        let stdin = child.stdin.take().context("failed to open stdin")?;
        let stdout = child.stdout.take().context("failed to open stdout")?;

        let (input_tx, input_rx) = mpsc::channel(32);
        tokio::spawn(async move {
            if let Err(e) = writer(stdin, input_rx).await {
                error!(cause = %e, "writer error");
            }
        });

        let (output_tx, output_rx) = mpsc::channel(32);
        tokio::spawn(async move {
            if let Err(e) = reader(stdout, output_tx).await {
                error!(cause = %e, "reader error");
            }
        });

        Ok(Self {
            id: 0,
            child,
            tx: input_tx,
            rx: output_rx,
            is_searching: false,
        })
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn wait(&mut self, keyword: &str) {
        while let Some(line) = self.rx.recv().await {
            if line == keyword {
                break;
            }
        }
    }

    pub async fn uci(&mut self) -> Result<()> {
        self.tx.send("uci".into()).await?;
        self.wait("uciok").await;
        Ok(())
    }

    pub async fn isready(&mut self) -> Result<()> {
        self.tx.send("isready".into()).await?;
        self.wait("readyok").await;
        Ok(())
    }

    pub async fn stop(&mut self) -> Result<()> {
        self.tx.send("stop\nisready".into()).await?;
        self.wait("readyok").await;
        Ok(())
    }

    pub async fn kill(&mut self) -> Result<()> {
        Ok(self.child.kill().await?)
    }

    pub async fn opts<O: std::fmt::Display>(&self, options: &[(O, O)]) -> Result<()> {
        let cmd = options.iter().fold(String::new(), |mut acc, (k, v)| {
            _ = writeln!(&mut acc, "setoption name {k} value {v}");
            acc
        });
        self.tx.send(cmd).await?;
        Ok(())
    }

    pub fn prepare(&self, job: Go) -> String {
        let mut cmd = "position".to_string();
        match &job.fen {
            None => _ = write!(&mut cmd, " startpos"),
            Some(fen) => _ = write!(&mut cmd, " fen {fen}"),
        };
        if !job.moves.is_empty() {
            _ = write!(&mut cmd, " moves {}", job.moves.join(" "));
        }
        cmd.push('\n');

        _ = writeln!(&mut cmd, "go depth {}", job.depth);

        cmd
    }

    pub async fn go(&mut self, job: Go) -> Result<(Info, BestMove)> {
        let cmd = self.prepare(job);
        self.tx.send(cmd).await?;

        let mut info: Option<Info> = None;
        let mut best: Option<BestMove> = None;

        while let Some(line) = self.rx.recv().await {
            match search(&line)? {
                Some(Search::Info(i)) => info = Some(i),
                Some(Search::BestMove(b)) => {
                    best = Some(b);
                    break;
                }
                None => continue,
            };
        }

        ensure!(info.is_some(), "failed to search for info");
        ensure!(best.is_some(), "failed to search for best move");

        Ok((info.unwrap(), best.unwrap()))
    }

    pub async fn go_with<V: Visitor>(&mut self, job: Go, visitor: &mut V) -> Result<()> {
        println!("go_with");
        let cmd = self.prepare(job);
        self.tx.send(cmd).await?;

        self.is_searching = true;
        while let Some(line) = self.rx.recv().await {
            match search(&line)? {
                Some(Search::Info(i)) => visitor.info(i),
                Some(Search::BestMove(b)) => {
                    visitor.best(b);
                    break;
                }
                None => continue,
            };
        }
        self.is_searching = false;

        Ok(())
    }

    pub async fn search(&mut self, job: Go, tx: mpsc::Sender<Search>) -> Result<()> {
        let cmd = self.prepare(job);
        self.tx.send(cmd).await?;

        while let Some(line) = self.rx.recv().await {
            match search(&line)? {
                Some(x) => tx.send(x).await?,
                None => continue,
            };
        }

        Ok(())
    }
}

#[derive(Debug, Default, Clone)]
pub struct Go {
    fen: Option<String>,
    moves: Vec<String>,
    depth: u32,
}

impl Go {
    pub fn new() -> Self {
        Self {
            depth: 10,
            ..Default::default()
        }
    }

    pub fn fen(mut self, fen: impl Into<String>) -> Self {
        self.fen = Some(fen.into());
        self
    }

    pub fn moves(mut self, moves: &[impl AsRef<str>]) -> Self {
        for mv in moves {
            self.moves.push(mv.as_ref().into());
        }
        self
    }

    pub fn depth(mut self, depth: u32) -> Self {
        self.depth = depth;
        self
    }

    pub async fn execute(self, engine: &mut Engine) -> Result<(Info, BestMove)> {
        engine.go(self).await
    }

    pub async fn execute_with(self, engine: &mut Engine, visitor: &mut impl Visitor) -> Result<()> {
        engine.go_with(self, visitor).await
    }
}

pub trait Visitor {
    fn info(&mut self, info: Info);
    fn best(&mut self, best: BestMove);
}

pub fn search(line: &str) -> Result<Option<Search>> {
    if line.starts_with("info depth") {
        let info = line.parse::<Info>()?;
        return Ok(Some(Search::Info(info)));
    }
    if line.starts_with("bestmove") {
        let best = line.parse::<BestMove>()?;
        return Ok(Some(Search::BestMove(best)));
    }
    Ok(None)
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Score {
    Cp(i32),
    Mate(i32),
}

impl Default for Score {
    fn default() -> Self {
        Self::Cp(0)
    }
}

impl Score {
    pub fn is_checkmate(&self) -> bool {
        matches!(self, Self::Mate(_))
    }
}

#[derive(Debug, Default, serde::Serialize)]
pub struct Info {
    /// The depth of the search, which is the number of half-moves the engine is looking ahead.
    pub depth: u32,
    /// The selective depth, which indicates the deepest point the search has reached in some lines.
    pub seldepth: u32,
    /// The number of principal variations (PVs) being considered. In this case, only the best move (single PV) is being reported.
    pub multipv: u32,
    /// The evaluation score of the position in centipawns (1/100th of a pawn). Shown from the side to move.
    pub score: Score,
    pub wdl: (u64, u64, u64),
    /// The number of positions (nodes) the engine has evaluated so far.
    pub nodes: u64,
    /// Nodes per second, which indicates the speed of the engine's search.
    pub nps: u64,
    /// The percentage of the hash table used.
    pub hashfull: u32,
    /// The number of times a position was found in the tablebases.
    pub tbhits: u64,
    /// The time in milliseconds the engine has spent on this search.
    pub time: u64,
    /// The principal variation, which is the sequence of moves the engine considers best from the current position.
    pub pv: Vec<String>,
}

#[derive(Debug, Default, serde::Serialize)]
pub struct BestMove {
    pub best: String,
    pub ponder: Option<String>,
}

#[derive(Debug)]
pub enum Search {
    Info(Info),
    BestMove(BestMove),
}

fn parse_info(line: &str) -> Result<Info> {
    let mut info = Info::default();
    let mut parts = line.split_whitespace();

    while let Some(part) = parts.next() {
        match part {
            "depth" => info.depth = parts.next().context("no depth")?.parse()?,
            "seldepth" => info.seldepth = parts.next().context("no seldepth")?.parse()?,
            "multipv" => info.multipv = parts.next().context("no multipv")?.parse()?,
            "score" => match parts.next().context("no score")? {
                "cp" => info.score = Score::Cp(parts.next().context("no cp")?.parse()?),
                "mate" => info.score = Score::Mate(parts.next().context("no mate")?.parse()?),
                other => bail!("Unknown score: {other}"),
            },
            "wdl" => {
                info.wdl.0 = parts.next().context("no win %")?.parse()?;
                info.wdl.1 = parts.next().context("no draw %")?.parse()?;
                info.wdl.2 = parts.next().context("no lose %")?.parse()?;
            }
            "nodes" => info.nodes = parts.next().context("no nodes")?.parse()?,
            "nps" => info.nps = parts.next().context("no nps")?.parse()?,
            "hashfull" => info.hashfull = parts.next().context("no hashfull")?.parse()?,
            "tbhits" => info.tbhits = parts.next().context("no tbhits")?.parse()?,
            "time" => info.time = parts.next().context("no time")?.parse()?,
            "pv" => {
                while let Some(mv) = parts.next() {
                    info.pv.push(mv.into());
                }
            }
            _ => (),
        };
    }

    Ok(info)
}

fn parse_bestmove(line: &str) -> Result<BestMove> {
    let parts = line.split_whitespace().collect::<Vec<_>>();
    Ok(BestMove {
        best: parts[1].into(),
        ponder: (parts.len() > 2).then(|| parts[3].into()),
    })
}

impl FromStr for Info {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        parse_info(s)
    }
}

impl FromStr for BestMove {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        parse_bestmove(s)
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_name() {
        assert_eq!(1, 1);
    }
}
