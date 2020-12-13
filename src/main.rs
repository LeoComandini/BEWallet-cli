use structopt::StructOpt;
use bewallet;

/// BEWallet Command Line Interface
#[derive(Debug, StructOpt)]
struct BEWalletCliOpt {
    /// The db path
    #[structopt(long)]
    data_root: String,
    
    /// Mnemonic
    #[structopt(long)]
    mnemonic: String,

    #[structopt(long)]
    electrum_url: String,

    #[structopt(long)]
    liquid: bool,

    #[structopt(long)]
    mainnet: bool,

    #[structopt(long)]
    development: bool,

    #[structopt(subcommand)]
    subcommand: BEWalletCliSubcommands,
}

#[derive(Debug, StructOpt)]
enum BEWalletCliSubcommands {
    SyncWallet,
    GetAddress,
    GetBalance,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = BEWalletCliOpt::from_args();

    let mut config = bewallet::Config::default();
    config.electrum_url = Some(args.electrum_url.clone());
    config.liquid = args.liquid;
    config.mainnet = args.mainnet;
    config.development = args.development;

    let wallet = bewallet::ElectrumWallet::new(config.clone(), &args.data_root, &args.mnemonic).unwrap();

    match args.subcommand {
        BEWalletCliSubcommands::SyncWallet => {
            println!("Sync: started");
            wallet.sync().unwrap();
            println!("Sync: done");
        },
        BEWalletCliSubcommands::GetAddress => {
            let ap = wallet.address().unwrap();
            println!("Address: {} (pointer: {})", ap.address, ap.pointer);
        }
        BEWalletCliSubcommands::GetBalance => {
            let balances = wallet.balance().unwrap();
            for (key, val) in balances.iter() {
                println!("{}: {}", key, val);
            }
        }
    }
    Ok(())
}
