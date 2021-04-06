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
    mainnet: bool,

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
    asset: String,
}

#[derive(Debug, StructOpt)]
enum BEWalletCliSubcommands {
    SyncWallet,
    GetAddress,
    GetBalance,
    GetTransactions,
    SendTransaction(SendOpt),
    GetCoins,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = BEWalletCliOpt::from_args();
    let spv_enabled = false;

    let wallet = if args.mainnet {
        let validate_domain = true;
        let tls = true;
        bewallet::ElectrumWallet::new_mainnet(
            &args.electrum_url,
            tls,
            validate_domain,
            spv_enabled,
            &args.data_root,
            &args.mnemonic,
        )
        .unwrap()
    } else {
        let validate_domain = false;
        let tls = false;
        bewallet::ElectrumWallet::new_regtest(
            &"TODO".to_string(),
            &args.electrum_url,
            tls,
            validate_domain,
            spv_enabled,
            &args.data_root,
            &args.mnemonic,
        )
        .unwrap()
    };

    match args.subcommand {
        BEWalletCliSubcommands::SyncWallet => {
            println!("Sync: started");
            wallet.sync().unwrap();
            println!("Sync: done");
        }
        BEWalletCliSubcommands::GetAddress => {
            let address = wallet.address().unwrap();
            println!("Address: {}", address.to_string());
        }
        BEWalletCliSubcommands::GetBalance => {
            let balances = wallet.balance().unwrap();
            for (key, val) in balances.iter() {
                println!("{}: {}", key, val);
            }
        }
        BEWalletCliSubcommands::GetTransactions => {
            let mut opt = bewallet::GetTransactionsOpt::default();
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
            let mut opt_create = bewallet::CreateTransactionOpt::default();
            opt_create.addressees.push(
                bewallet::Destination::new(&opt_send.address, opt_send.satoshi, &opt_send.asset)
                    .unwrap(),
            );
            let mut tx = wallet.create_tx(&mut opt_create).unwrap().transaction;
            wallet.sign_tx(&mut tx, &args.mnemonic).unwrap();
            wallet.broadcast_tx(&tx).unwrap();
            println!("txid: {}", tx.txid());
        }
        BEWalletCliSubcommands::GetCoins => {
            for utxo in wallet.utxos().unwrap() {
                println!(
                    "outpoint {}:{}",
                    utxo.txo.outpoint.txid.to_string(),
                    utxo.txo.outpoint.vout
                );
                println!("  satoshi: {}", utxo.unblinded.value);
                println!("  asset:   {}", utxo.unblinded.asset);
            }
        }
    }
    Ok(())
}
