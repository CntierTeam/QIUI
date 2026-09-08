#![allow(dead_code)]

mod api;
mod ble;
mod config;
mod crypto;
mod protocol;

use api::{Client, DEFAULT_BASE};
use clap::{Parser, Subcommand};
use config::Config;
use protocol::Profile;
use tracing_subscriber::EnvFilter;

#[derive(Parser, Debug)]
#[command(name = "qiui", about = "Independent QIUI/Cellmate cloud API + BLE control CLI")]
struct Cli {
    #[arg(long, global = true, env = "QIUI_BASE")]
    base: Option<String>,

    #[arg(long, global = true, env = "QIUI_TOKEN")]
    token: Option<String>,

    #[arg(long, global = true)]
    verbose: bool,

    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand, Debug)]
enum Cmd {
    /// Fetch & decrypt dynamic API host list
    DiscoverApi,
    /// Password login → save token to ~/.config/qiui/config.toml
    Login {
        #[arg(long, short)]
        user: String,
        #[arg(long, short)]
        password: String,
        /// phone+password (loginType=3); default email+password (loginType=2)
        #[arg(long)]
        phone: bool,
    },
    /// Show saved token / identity
    Whoami,
    /// List bound toys (`getUserBindingToyDevices`)
    Devices,
    /// Ask cloud for Cellmate BLE write hex (getToyToken)
    GetToyToken {
        #[arg(long)]
        toy_uid: String,
    },
    /// Ask cloud for Cellmate close/lock BLE hex
    CloseLock {
        #[arg(long)]
        toy_uid: String,
    },
    /// Decrypt notify hex via cloud `decryBluetoothCommand`
    DecryNotify {
        #[arg(long)]
        hex: String,
    },
    /// Local crypto helpers
    #[command(subcommand)]
    Crypto(CryptoCmd),
    /// List supported BLE GATT profiles
    Profiles,
    /// BLE scan
    Scan {
        #[arg(long, default_value_t = 5)]
        seconds: u64,
        #[arg(long, default_value = "cellmate")]
        profile: String,
    },
    /// BLE connect + write hex (+ optional wait notify)
    Write {
        #[arg(long)]
        address: String,
        #[arg(long)]
        hex: String,
        #[arg(long, default_value = "cellmate")]
        profile: String,
        #[arg(long, default_value_t = 3000)]
        wait_ms: u64,
        #[arg(long)]
        mock: bool,
    },
}

#[derive(Subcommand, Debug)]
enum CryptoCmd {
    EncryptApi { text: String },
    DecryptApi { b64: String },
    EncryptPwd { text: String },
    EncryptBt { text: String },
    DecryptBt { b64: String },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let filter = if cli.verbose {
        EnvFilter::new("debug")
    } else {
        EnvFilter::new("info")
    };
    tracing_subscriber::fmt().with_env_filter(filter).init();

    match cli.cmd {
        Cmd::DiscoverApi => {
            let (c, eps) = Client::discover()?;
            println!("base={}", c.base);
            println!("http_url={}", eps.http_url);
            println!("http_image={}", eps.http_image);
            println!("http_chat={}", eps.http_chat);
            let mut cfg = Config::load().unwrap_or_default();
            cfg.base_url = Some(c.base);
            cfg.save()?;
        }
        Cmd::Login {
            ref user,
            ref password,
            phone,
        } => {
            let mut client = make_client(&cli)?;
            let login_type = if phone { "3" } else { "2" };
            let data = client.login_password(user, password, login_type)?;
            println!("state=ok");
            println!("token={}", data.token.as_deref().unwrap_or(""));
            println!("uid={}", data.uid.as_deref().unwrap_or(""));
            println!("userId={}", data.user_id.as_deref().unwrap_or(""));
            println!("nickname={}", data.nickname.as_deref().unwrap_or(""));
            let mut cfg = Config::load().unwrap_or_default();
            cfg.base_url = Some(client.base.clone());
            cfg.token = data.token.clone();
            cfg.uid = data.uid.clone();
            cfg.user_id = data.user_id.clone();
            cfg.nickname = data.nickname.clone();
            cfg.user_name = Some(user.clone());
            cfg.save()?;
            println!("saved={}", Config::path().display());
        }
        Cmd::Whoami => {
            let cfg = Config::load()?;
            println!("config={}", Config::path().display());
            println!("base={}", cfg.base_url.as_deref().unwrap_or(""));
            println!("token={}", cfg.token.as_deref().unwrap_or(""));
            println!("uid={}", cfg.uid.as_deref().unwrap_or(""));
            println!("userId={}", cfg.user_id.as_deref().unwrap_or(""));
            println!("nickname={}", cfg.nickname.as_deref().unwrap_or(""));
            println!("user={}", cfg.user_name.as_deref().unwrap_or(""));
        }
        Cmd::Devices => {
            let client = make_authed(&cli)?;
            let data = client.get_binding_devices()?;
            println!("{}", serde_json::to_string_pretty(&data)?);
        }
        Cmd::GetToyToken { ref toy_uid } => {
            let client = make_authed(&cli)?;
            let hex = client.cellmate_get_toy_token(toy_uid)?;
            println!("{hex}");
        }
        Cmd::CloseLock { ref toy_uid } => {
            let client = make_authed(&cli)?;
            let hex = client.cellmate_close_lock(toy_uid)?;
            println!("{hex}");
        }
        Cmd::DecryNotify { ref hex } => {
            let client = make_authed(&cli)?;
            let data = client.cellmate_decry_command(hex)?;
            println!("{}", serde_json::to_string_pretty(&data)?);
        }
        Cmd::Crypto(c) => match c {
            CryptoCmd::EncryptApi { text } => println!("{}", crypto::encrypt_api(&text)?),
            CryptoCmd::DecryptApi { b64 } => println!("{}", crypto::decrypt_api(&b64)?),
            CryptoCmd::EncryptPwd { text } => println!("{}", crypto::encrypt_password(&text)?),
            CryptoCmd::EncryptBt { text } => println!("{}", crypto::encrypt_bt_command(&text)?),
            CryptoCmd::DecryptBt { b64 } => println!("{}", crypto::decrypt_bt_command(&b64)?),
        },
        Cmd::Profiles => {
            for p in Profile::all() {
                println!(
                    "{}\tservice={}\twrite={}\tnotify={}",
                    p.name(),
                    p.service(),
                    p.write(),
                    p.notify()
                );
            }
        }
        Cmd::Scan { seconds, profile } => {
            let rt = tokio::runtime::Runtime::new()?;
            rt.block_on(async move {
                let profile = parse_profile(&profile)?;
                let s = ble::Session::open(profile).await?;
                let found = s.scan(seconds).await?;
                for (addr, name) in found {
                    println!("{addr}\t{}", name.unwrap_or_default());
                }
                Ok::<(), anyhow::Error>(())
            })?;
        }
        Cmd::Write {
            address,
            hex,
            profile,
            wait_ms,
            mock,
        } => {
            let profile = parse_profile(&profile)?;
            if mock {
                let mut m = ble::MockSession::new(profile);
                let n = m.write_hex(&hex)?;
                println!("wrote={}", hex);
                println!("notify_mock={}", hex::encode(n));
            } else {
                let rt = tokio::runtime::Runtime::new()?;
                rt.block_on(async move {
                    let mut s = ble::Session::open(profile).await?;
                    s.connect(&address).await?;
                    s.write_hex(&hex).await?;
                    println!("wrote={}", hex);
                    if wait_ms > 0 {
                        match s.wait_notify(wait_ms).await {
                            Ok(n) => println!("notify={}", hex::encode(n)),
                            Err(e) => eprintln!("notify: {e}"),
                        }
                    }
                    Ok::<(), anyhow::Error>(())
                })?;
            }
        }
    }
    Ok(())
}

fn parse_profile(s: &str) -> anyhow::Result<Profile> {
    Profile::parse(s).ok_or_else(|| anyhow::anyhow!("unknown profile `{s}` (see `qiui profiles`)"))
}

fn make_client(cli: &Cli) -> anyhow::Result<Client> {
    let cfg = Config::load().unwrap_or_default();
    let base = cli
        .base
        .clone()
        .or(cfg.base_url)
        .unwrap_or_else(|| DEFAULT_BASE.to_string());
    let mut c = Client::new(base)?;
    if let Some(t) = cli.token.clone().or(cfg.token) {
        c = c.with_token(t);
    }
    Ok(c)
}

fn make_authed(cli: &Cli) -> anyhow::Result<Client> {
    let c = make_client(cli)?;
    if c.token.is_none() {
        anyhow::bail!("no token; run `qiui login` or pass --token / QIUI_TOKEN");
    }
    Ok(c)
}
