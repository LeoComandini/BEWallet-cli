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

    #[structopt(long)]
    testnet: bool,

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
struct LiquidexMakeOpt {
    #[structopt(long)]
    txid: String,

    #[structopt(long)]
    vout: u32,

    #[structopt(long)]
    asset: String,

    #[structopt(long)]
    rate: f64,
}

#[derive(Debug, StructOpt)]
struct LiquidexTakeOpt {
    #[structopt(long)]
    proposal: String,

    #[structopt(long)]
    broadcast: bool,
}

#[derive(Debug, StructOpt)]
enum BEWalletCliSubcommands {
    SyncWallet,
    GetAddress,
    GetBalance,
    GetTransactions,
    SendTransaction(SendOpt),
    GetCoins,
    LiquidexMake(LiquidexMakeOpt),
    LiquidexTake(LiquidexTakeOpt),
}

fn main() -> Result<(), bewallet::Error> {
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
        )?
    } else if args.testnet {
        let validate_domain = true;
        let tls = true;
        bewallet::ElectrumWallet::new_testnet(
            &args.electrum_url,
            tls,
            validate_domain,
            spv_enabled,
            &args.data_root,
            &args.mnemonic,
        )?
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
        )?
    };

    match args.subcommand {
        BEWalletCliSubcommands::SyncWallet => {
            wallet.sync()?;
        }
        BEWalletCliSubcommands::GetAddress => {
            let address = wallet.address()?;
            println!("{}", address.to_string());
        }
        BEWalletCliSubcommands::GetBalance => {
            let balances = wallet.balance()?;
            println!("{}", serde_json::to_string(&balances).unwrap());
        }
        BEWalletCliSubcommands::GetTransactions => {
            let mut opt = bewallet::GetTransactionsOpt::default();
            opt.count = 100;
            let transactions = wallet.transactions(&opt)?;
            println!("{}", serde_json::to_string(&transactions).unwrap());
        }
        BEWalletCliSubcommands::SendTransaction(opt_send) => {
            let mut opt_create = bewallet::CreateTransactionOpt::default();
            opt_create.addressees.push(bewallet::Destination::new(
                &opt_send.address,
                opt_send.satoshi,
                &opt_send.asset,
            )?);
            let mut tx = wallet.create_tx(&mut opt_create)?.transaction;
            wallet.sign_tx(&mut tx, &args.mnemonic)?;
            wallet.broadcast_tx(&tx)?;
            println!("{}", tx.txid());
        }
        BEWalletCliSubcommands::GetCoins => {
            let utxos = wallet.utxos()?;
            println!("{}", serde_json::to_string(&utxos).unwrap());
        }
        BEWalletCliSubcommands::LiquidexMake(opt) => {
            let opt = bewallet::LiquidexMakeOpt::new(&opt.txid, opt.vout, &opt.asset, opt.rate)?;

            // Insert asset in local db so the wallet can receive it
            wallet.liquidex_assets_insert(opt.asset_id)?;

            let proposal = wallet.liquidex_make(&opt, &args.mnemonic)?;
            println!("{}", serde_json::to_string(&proposal)?);
        }
        BEWalletCliSubcommands::LiquidexTake(opt) => {
            let proposal: bewallet::LiquidexProposal = serde_json::from_str(&opt.proposal)?;
            let tx = wallet.liquidex_take(&proposal, &args.mnemonic)?;
            if opt.broadcast {
                wallet.broadcast_tx(&tx)?;
                println!("{}", tx.txid());
            } else {
                println!("{}", bewallet::tx_to_hex(&tx));
            }
        }
    }
    Ok(())
}
