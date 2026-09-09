#![allow(dead_code)]

mod api;
mod ble;
mod config;
mod crypto;
mod product;
mod protocol;

use api::{Client, DEFAULT_BASE};
use clap::{Parser, Subcommand};
use config::Config;
use protocol::Profile;
use tracing_subscriber::EnvFilter;

#[derive(Parser, Debug)]
#[command(
    name = "qiui",
    about = "QIUI product CLI: cloud login + BLE control for Cellmate / KeyPod / PearFlower / collar / …"
)]
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
    /// List product families + actions (Cellmate / KeyPod / 项圈 / …)
    Products,
    /// Run a product cloud action → BLE hex (or JSON for decry/collar)
    Run {
        /// Product id from `qiui products` (e.g. cellmate, keypod-metal, collar)
        #[arg(long, short = 'p')]
        product: String,
        /// Action name (token, lock, unlock, shock, …)
        #[arg(long, short = 'a')]
        action: String,
        #[arg(long)]
        toy_uid: String,
        /// Required for `decry` / collar `unlock`/`decrypt`
        #[arg(long)]
        hex: Option<String>,
    },
    /// Ask cloud for Cellmate BLE write hex (alias: run -p cellmate -a token)
    GetToyToken {
        #[arg(long)]
        toy_uid: String,
    },
    /// Ask cloud for Cellmate close/lock BLE hex (alias: run -p cellmate -a close-lock)
    CloseLock {
        #[arg(long)]
        toy_uid: String,
    },
    /// Decrypt notify hex via Cellmate cloud (alias: run -p cellmate -a decry --hex)
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
        /// Optional product id → pick default BLE profile
        #[arg(long, short = 'p')]
        product: Option<String>,
    },
    /// BLE connect + write hex (+ optional wait notify)
    Write {
        #[arg(long)]
        address: String,
        #[arg(long)]
        hex: String,
        #[arg(long, default_value = "cellmate")]
        profile: String,
        /// Optional product id → pick default BLE profile (overrides --profile)
        #[arg(long, short = 'p')]
        product: Option<String>,
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
        Cmd::Products => {
            for p in product::all() {
                println!(
                    "{}\t{}\tprofile={}\t({})",
                    p.id,
                    p.title_zh,
                    p.profile.name(),
                    p.title_en
                );
                for a in p.actions {
                    println!("  - {}\t{}", a.name, a.summary);
                }
            }
        }
        Cmd::Run {
            ref product,
            ref action,
            ref toy_uid,
            ref hex,
        } => {
            let client = make_authed(&cli)?;
            run_product_cmd(&client, product, action, toy_uid, hex.as_deref())?;
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
        Cmd::Scan {
            seconds,
            profile,
            product: prod,
        } => {
            let profile = resolve_profile(prod.as_deref(), &profile)?;
            let rt = tokio::runtime::Runtime::new()?;
            rt.block_on(async move {
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
            product: prod,
            wait_ms,
            mock,
        } => {
            let profile = resolve_profile(prod.as_deref(), &profile)?;
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

fn run_product_cmd(
    client: &Client,
    product_id: &str,
    action_name: &str,
    toy_uid: &str,
    hex: Option<&str>,
) -> anyhow::Result<()> {
    let prod = product::get(product_id).ok_or_else(|| {
        anyhow::anyhow!("unknown product `{product_id}` (see `qiui products`)")
    })?;
    let act = product::find_action(prod, action_name).ok_or_else(|| {
        anyhow::anyhow!(
            "unknown action `{action_name}` for `{product_id}` (see `qiui products`)"
        )
    })?;

    if act.name == "decry" {
        let h = hex.ok_or_else(|| anyhow::anyhow!("`--hex` required for action `decry`"))?;
        let data = client.bt_decry(act.path, h)?;
        println!("{}", serde_json::to_string_pretty(&data)?);
        return Ok(());
    }

    if prod.id == "collar" {
        let h = hex.ok_or_else(|| {
            anyhow::anyhow!("`--hex` required for collar actions (local/notify command hex)")
        })?;
        let data = client.collar_record_cmd(act.path, toy_uid, h)?;
        match data {
            serde_json::Value::String(s) => println!("{s}"),
            other => println!("{}", serde_json::to_string_pretty(&other)?),
        }
        return Ok(());
    }

    let out = client.bt_cmd_toy_uid(act.path, toy_uid)?;
    println!("{out}");
    eprintln!(
        "# product={} action={} profile={} → qiui write -p {} --address <MAC> --hex …",
        prod.id,
        act.name,
        prod.profile.name(),
        prod.id
    );
    Ok(())
}

fn resolve_profile(product_id: Option<&str>, profile: &str) -> anyhow::Result<Profile> {
    if let Some(pid) = product_id {
        let p = product::get(pid)
            .ok_or_else(|| anyhow::anyhow!("unknown product `{pid}` (see `qiui products`)"))?;
        return Ok(p.profile);
    }
    parse_profile(profile)
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
