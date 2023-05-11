use clap::{arg, Command};
use rchain::{Blockchain, ProofOfWork};
use std::env::current_dir;

fn main() {
    env_logger::init();
    let matches = cli().get_matches();
    match matches.subcommand() {
        Some(("ls", _)) => {
            let path = current_dir().unwrap();
            let address = path.file_name().expect("filename").to_str().expect("file");
            let chain = Blockchain::new(&path, address.to_owned()).unwrap();


            print_chain(chain);
        }
        Some(("add", sub_matches)) => {
            let path = current_dir().unwrap();
            let address = path.file_name().expect("filename").to_str().expect("file");
            let mut chain = Blockchain::new(&path, address.to_owned()).unwrap();

            // let data = sub_matches.get_one::<String>("DATA").expect("require");
            // chain.add_block(data.to_owned()).expect("err");

            print_chain(chain);
        },
        Some(("balance", sub_matches)) => {
            let user = sub_matches.get_one::<String>("ADDRESS").expect("address");

            let path = current_dir().unwrap();
            let address = path.file_name().expect("filename").to_str().expect("file");
            let chain = Blockchain::new(&path, address.to_owned()).unwrap();

            let mut balance  = 0;
            let utxo = chain.find_utxo(user.to_owned());
            for (_, v) in utxo {
                balance += v.iter().fold(0, |acc, x| acc + x.value);
            }
            println!("balance: {}", balance);
        },
        _ => panic!("no implemented"),
    }
}

fn print_chain(chain: Blockchain) {
    let iter = chain.into_iter();
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
            Command::new("add")
                .about("add a new block to blockchain")
                .arg_required_else_help(true)
                .arg(arg!([DATA] "data")),
        )
        .subcommand(
            Command::new("balance")
                .about("show the balance of the address.")
                .arg_required_else_help(true)
                .arg(arg!([ADDRESS] "address"))
        )
}
