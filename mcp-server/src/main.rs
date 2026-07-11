#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

mod auth;
mod http;
mod protocol;
mod tools;

use anyhow::{bail, Context, Result};
use neopad_core::init_workspace;
use std::net::IpAddr;
use std::path::PathBuf;

#[derive(Debug, Clone)]
struct Args {
    command: Command,
    workspace: Option<PathBuf>,
}

#[derive(Debug, Clone)]
enum Command {
    Serve(http::ServerOptions),
}

#[tokio::main]
async fn main() {
    if let Err(error) = run().await {
        eprintln!("neopad-mcp error: {error:#}");
        std::process::exit(1);
    }
}

async fn run() -> Result<()> {
    let args = parse_args(std::env::args().skip(1))?;
    let workspace = init_workspace(args.workspace)?;

    match args.command {
        Command::Serve(options) => http::serve(workspace, options).await,
    }
}

fn parse_args(args: impl Iterator<Item = String>) -> Result<Args> {
    let mut workspace = None;
    let mut command = None;
    let mut args = args.peekable();

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "serve" => {
                if command.is_some() {
                    bail!("only one command may be provided");
                }
                command = Some(Command::Serve(parse_serve_args(&mut args)?));
                break;
            }
            "--workspace" => {
                let value = args.next().context("--workspace requires a path")?;
                workspace = Some(expand_home_path(&value));
            }
            "--help" | "-h" => {
                print_help();
                std::process::exit(0);
            }
            "--allow-write" => {
                bail!("stdio and --allow-write are no longer supported; use `neopad-mcp serve --token <token>`");
            }
            _ => bail!("unknown argument: {arg}"),
        }
    }

    let command = command.context("missing command: expected `serve`")?;
    Ok(Args { command, workspace })
}

fn parse_serve_args(args: &mut impl Iterator<Item = String>) -> Result<http::ServerOptions> {
    let mut host = IpAddr::from([127, 0, 0, 1]);
    let mut port = 8765;
    let mut token = None;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--host" => {
                let value = args.next().context("--host requires an address")?;
                host = value
                    .parse()
                    .with_context(|| format!("invalid host address: {value}"))?;
            }
            "--port" => {
                let value = args.next().context("--port requires a number")?;
                port = value
                    .parse()
                    .with_context(|| format!("invalid port: {value}"))?;
            }
            "--token" => {
                token = Some(args.next().context("--token requires a value")?);
            }
            "--help" | "-h" => {
                print_help();
                std::process::exit(0);
            }
            _ => bail!("unknown serve argument: {arg}"),
        }
    }

    let token = token
        .or_else(|| std::env::var("NEOPAD_MCP_TOKEN").ok())
        .filter(|value: &String| !value.trim().is_empty())
        .context("--token or NEOPAD_MCP_TOKEN is required for HTTP MCP")?;

    if !host.is_loopback() {
        bail!("MCP HTTP host must be a loopback address");
    }

    Ok(http::ServerOptions { host, port, token })
}

fn expand_home_path(path: &str) -> PathBuf {
    if path == "~" {
        return dirs::home_dir().unwrap_or_else(|| PathBuf::from(path));
    }
    if let Some(rest) = path.strip_prefix("~/") {
        if let Some(home) = dirs::home_dir() {
            return home.join(rest);
        }
    }
    PathBuf::from(path)
}

fn print_help() {
    eprintln!(
        "Usage: neopad-mcp [--workspace <path>] serve [--host 127.0.0.1] [--port 8765] --token <token>"
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serve_rejects_non_loopback_hosts() {
        let args = ["serve", "--host", "0.0.0.0", "--token", "secret"]
            .into_iter()
            .map(str::to_owned);
        assert!(parse_args(args).is_err());
    }

    #[test]
    fn serve_accepts_ipv4_and_ipv6_loopback_hosts() {
        for host in ["127.0.0.1", "::1"] {
            let args = ["serve", "--host", host, "--token", "secret"]
                .into_iter()
                .map(str::to_owned);
            assert!(parse_args(args).is_ok(), "{host} should be accepted");
        }
    }
}
