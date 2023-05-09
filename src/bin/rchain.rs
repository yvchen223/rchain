use clap::{arg, Command};
use rchain::{Blockchain, ProofOfWork};
use std::env::current_dir;

fn main() {
    env_logger::init();
    let matches = cli().get_matches();
    match matches.subcommand() {
        Some(("ls", _)) => {
            let path = current_dir().unwrap();
            let chain = Blockchain::new(path).unwrap();

            print_chain(chain);
        }
        Some(("add", sub_matches)) => {
            let path = current_dir().unwrap();
            let mut chain = Blockchain::new(path).unwrap();

            let data = sub_matches.get_one::<String>("DATA").expect("require");
            chain.add_block(data.to_owned()).expect("err");

            print_chain(chain);
        }
        _ => panic!("no implemented"),
    }
}

fn print_chain(chain: Blockchain) {
    let iter = chain.into_iter();
    for block in iter {
        println!("pre_hash: {}", block.pre_hash);
        println!("hash: {}", block.hash);
        println!("data: {}", block.data);
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
}
