use crate::commands::OutputFormat;
use anyhow::{Context, Result};
use pass::PassClient;
use pass_domain::{ItemId, ShareId};

#[derive(serde::Serialize)]
struct JsonAliasItem {
    id: ItemId,
    alias: String,
}

pub async fn run(
    client: PassClient,
    share_id: ShareId,
    prefix: String,
    output: OutputFormat,
) -> Result<()> {
    let res = client
        .create_alias(&share_id, &prefix)
        .await
        .context("Error creating alias")?;

    match output {
        OutputFormat::Human => {
            println!("{}", res.alias);
        }
        OutputFormat::Json => {
            let res = JsonAliasItem {
                id: res.item_id,
                alias: res.alias,
            };
            println!(
                "{}",
                serde_json::to_string_pretty(&res).context("Error serializing output")?
            );
        }
    }

    Ok(())
}
