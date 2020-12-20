use bewallet;
use structopt::StructOpt;

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
struct SendOpt {
    #[structopt(long)]
    address: String,

    #[structopt(long)]
    satoshi: u64,

    #[structopt(long)]
    asset: Option<String>,
}

#[derive(Debug, StructOpt)]
enum BEWalletCliSubcommands {
    SyncWallet,
    GetAddress,
    GetBalance,
    GetTransactions,
    SendTransaction(SendOpt),
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = BEWalletCliOpt::from_args();

    let mut config = bewallet::Config::default();
    config.electrum_url = Some(args.electrum_url.clone());
    config.tls = Some(!args.development);
    config.liquid = args.liquid;
    config.mainnet = args.mainnet;
    config.development = args.development;

    let wallet =
        bewallet::ElectrumWallet::new(config.clone(), &args.data_root, &args.mnemonic).unwrap();

    match args.subcommand {
        BEWalletCliSubcommands::SyncWallet => {
            println!("Sync: started");
            wallet.sync().unwrap();
            println!("Sync: done");
        }
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
        BEWalletCliSubcommands::GetTransactions => {
            let mut opt = bewallet::model::GetTransactionsOpt::default();
            opt.count = 100;
            let transactions = wallet.transactions(&opt).unwrap();
            for transaction in transactions.iter() {
                println!("txid: {}", transaction.txid);
                for (key, val) in transaction.balances.iter() {
                    println!("  {}: {}", key, val);
                }
            }
        }
        BEWalletCliSubcommands::SendTransaction(opt_send) => {
            let mut opt_create = bewallet::model::CreateTransactionOpt::default();
            opt_create.addressees.push(bewallet::model::AddressAmount {
                address: opt_send.address,
                satoshi: opt_send.satoshi,
                asset_tag: opt_send.asset,
            });
            let mut tx = wallet.create_tx(&mut opt_create).unwrap().transaction;
            wallet.sign_tx(&mut tx, &args.mnemonic).unwrap();
            wallet.broadcast_tx(&tx).unwrap();
            println!("txid: {}", tx.txid());
        }
    }
    Ok(())
}
