use clap::{arg, Arg, Command};
use rchain::{Blockchain, ProofOfWork, Transaction};
use std::env::current_dir;
use rchain::wallet::Wallet;

fn main() {
    env_logger::init();
    let path = current_dir().unwrap();
    let init_address = "1FbkP5rheSAtFonCjoNikSofyrGNHMUqzA";
    let mut chain = Blockchain::new(&path, init_address.to_owned()).unwrap();
    let matches = cli().get_matches();
    match matches.subcommand() {
        Some(("ls", _)) => {

            print_chain(&chain);
        }
        Some(("balance", sub_matches)) => {
            let user = sub_matches.get_one::<String>("ADDRESS").expect("address");

            let mut balance = 0;
            let utxo = chain.find_utxo(user);
            for (_, v) in utxo {
                balance += v.iter().fold(0, |acc, (_, x)| acc + x.value);
            }
            println!("balance: {}", balance);
        }
        Some(("send", sub_match)) => {
            let from = sub_match.get_one::<String>("FROM").expect("from");
            let to = sub_match.get_one::<String>("TO").expect("to");
            let amount: i64 = *sub_match.get_one::<i64>("AMOUNT").expect("amount");

            let tx = Transaction::new(from, to, amount, &chain).unwrap();
            chain.mine_block(vec![tx]).unwrap();

            print_chain(&chain);
        }
        Some(("create-wallet", _)) => {
            let wallet = Wallet::new();
            let address = wallet.address();
            println!("wallet: {:?}", wallet);
            println!("address: {}", address);
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
        .subcommand(
            Command::new("create-wallet")
                .about("create a wallet.")
        )
}
