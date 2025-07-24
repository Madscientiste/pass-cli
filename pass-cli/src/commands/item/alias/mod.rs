mod create;

use crate::commands::OutputFormat;
use anyhow::Result;
use clap::Subcommand;
use pass::PassClient;
use pass_domain::ShareId;

#[derive(Subcommand)]
pub enum AliasCommands {
    #[command(about = "Create a new alias")]
    Create {
        #[arg(long, help = "Share ID of the vault where the alias will be created")]
        share_id: String,
        #[arg(
            long,
            help = "Prefix of the alias. The resulting email will be [prefix].[suffix]"
        )]
        prefix: String,
        #[arg(long, help = "Output format", default_value = "human")]
        output: OutputFormat,
    },
}

pub async fn run(subcommand: AliasCommands, client: PassClient) -> Result<()> {
    match subcommand {
        AliasCommands::Create {
            share_id,
            prefix,
            output,
        } => create::run(client, ShareId::new(share_id), prefix, output).await,
    }
}
