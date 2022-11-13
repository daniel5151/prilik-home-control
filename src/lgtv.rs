use anyhow::Context;
use clap::Parser;
use clap::Subcommand;
use lg_webos_client::client::*;
use lg_webos_client::command::Command;

/// Control my LG WebOS TV
#[derive(Parser)]
pub struct LgTv {
    #[clap(long)]
    ip: String,

    #[clap(long)]
    mac: Option<String>,

    #[clap(long)]
    webos_key: Option<String>,

    #[clap(subcommand)]
    action: LgAction,
}

#[derive(Clone, Subcommand)]
enum LgAction {
    /// Turn the TV on
    On,
    /// Turn the TV off
    Off,
}

impl LgTv {
    pub async fn handle(self) -> anyhow::Result<()> {
        // turning on the TV goes through an entirely different flow, since it
        // uses a wake-on-lan packet
        if let LgAction::On = self.action {
            let Some(mac) = self.mac else {
                anyhow::bail!("must pass mac address for turn-on")
            };

            let mac = mac
                .parse::<wol::MacAddr>()
                .map_err(anyhow::Error::msg)
                .context("could not parse mac address")?;

            // really hammer it with packets - it's very temperamental
            for _ in 0..10 {
                wol::send_wol(mac, None, None).context("failed to send wake-on-lan packet")?;
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            }

            println!("sent wake-on-lan packet to {mac}");

            return Ok(());
        }

        let requires_auth = self.webos_key.is_none();

        // if no key was found, then we need to do through the initial auth flow with
        // the TV
        if requires_auth {
            println!("no webos-key provided: doing initial auth flow");
            println!("follow the steps on the TV");
        }

        let config = WebOsClientConfig::new(&format!("ws://{}:3000/", self.ip), self.webos_key);
        let client = WebosClient::new(config)
            .await
            .map_err(anyhow::Error::msg)
            .context("could not create WebosClient")?;

        // rather than silently completing the op, force the user to properly pass the
        // key and re-run the cmd
        if requires_auth {
            println!("webos-key: {}", client.key.unwrap());
            return Ok(());
        }

        match self.action {
            LgAction::On => unreachable!(),
            LgAction::Off => {
                let resp = client.send_command(Command::TurnOff).await.unwrap();
                println!("Got response {:?}", resp.payload);
            }
        }

        Ok(())
    }
}
