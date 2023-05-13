use clap::{arg, Arg, Command};
use rchain::wallet::{Wallet, Wallets};
use rchain::{Blockchain, ProofOfWork, Transaction};
use std::env::current_dir;

fn main() {
    env_logger::init();
    let path = current_dir().unwrap();
    let matches = cli().get_matches();
    match matches.subcommand() {
        Some(("create-blockchain", sub_matches)) => {
            let address = sub_matches.get_one::<String>("ADDRESS").expect("address");
            let chain = Blockchain::new(&path, address).unwrap();
            print_chain(&chain);
        }
        Some(("ls", _)) => {
            let address;
            {
                let wallets = Wallets::with_path(&path);
                let wallet = Wallet::new();
                address = wallet.address();
                wallets.set(&wallet).unwrap();
            }
            let chain = Blockchain::new(&path, &address).unwrap();
            print_chain(&chain);
        }
        Some(("balance", sub_matches)) => {
            let user = sub_matches.get_one::<String>("ADDRESS").expect("address");

            let chain = Blockchain::new(&path, user).unwrap();

            let mut balance = 0;
            let pub_key_hash = Transaction::pub_key_hash_from_address(user);
            let utxo = chain.find_utxo(&pub_key_hash);
            for (_, v) in utxo {
                balance += v.iter().fold(0, |acc, (_, x)| acc + x.value);
            }
            println!("balance: {}", balance);
        }
        Some(("send", sub_match)) => {
            let from = sub_match.get_one::<String>("FROM").expect("from");
            let to = sub_match.get_one::<String>("TO").expect("to");
            let amount: i64 = *sub_match.get_one::<i64>("AMOUNT").expect("amount");

            let mut chain = Blockchain::new(&path, from).unwrap();

            let tx = Transaction::new(from, to, amount, &chain).unwrap();
            chain.mine_block(vec![tx]).unwrap();

            print_chain(&chain);
        }
        Some(("create-wallet", _)) => {
            let wallets = Wallets::with_path(&path);
            let wallet = Wallet::new();
            let address = wallet.address();
            println!("wallet: {:?}", wallet);
            println!("address: {}", address);
            wallets.set(&wallet).unwrap();
        }
        Some(("wallets", _)) => {
            let wallets = Wallets::with_path(&path);
            let v = wallets.list();
            for (address, w) in &v {
                println!("addr: {}, wallet: {:?}", address, w);
            }
        }
        _ => panic!("no implemented"),
    }
}

fn print_chain(chain: &Blockchain) {
    let iter = chain.iter();
    for block in iter {
        println!("pre_hash: {}", block.pre_hash);
        println!("hash: {}", block.hash);
        println!("transaction: {:?}", block.transactions);
        println!("nonce: {}", block.nonce);
        println!("timestamp: {}", block.timestamp);
        let pow = ProofOfWork::new(block.clone());
        println!("pow: {}", pow.validate());
        println!();
    }
}

fn cli() -> Command {
    Command::new("rchain")
        .about("A blockchain implemented in Rust")
        .author("yvchen223@gmail.com")
        .version(env!("CARGO_PKG_VERSION"))
        .long_version(env!("CARGO_PKG_VERSION"))
        //.subcommand_required(true)
        .allow_external_subcommands(true)
        .arg_required_else_help(true)
        .subcommand(Command::new("ls").about("list the blockchain store in this directory"))
        .subcommand(
            Command::new("balance")
                .about("show the balance of the address.")
                .arg_required_else_help(true)
                .arg(arg!([ADDRESS] "address")),
        )
        .subcommand(
            Command::new("send")
                .about("send coins from someone to another.")
                .arg_required_else_help(true)
                .args([
                    arg!([FROM] "from"),
                    arg!([TO] "to"),
                    Arg::new("AMOUNT").value_parser(clap::value_parser!(i64)),
                ]),
        )
        .subcommand(Command::new("create-wallet").about("create a wallet."))
        .subcommand(Command::new("wallets"))
        .subcommand(
            Command::new("create-blockchain")
                .about("Create a blockchain.")
                .arg_required_else_help(true)
                .arg(arg!([ADDRESS] "address")),
        )
}
