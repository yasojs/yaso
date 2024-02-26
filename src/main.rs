use yaso::cli::{Command, CLI};
use yaso::vm::VirtualMachine;
use yaso::STARTING_TIME;

use clap::Parser;

use std::process::exit;
use std::time::Instant;


#[tokio::main]
pub async fn main() {
    let cli = CLI::parse();

    match cli.command {
        Command::Run { file_path, .. } => {
            if !file_path.exists() {
                eprintln!("{}: No such file or directory", file_path.display());

                exit(1);
            }

            unsafe { STARTING_TIME.write(Instant::now()) };

            let vm = VirtualMachine::new().await;

            vm.init().await;

            vm.run_module(&file_path).await;

            vm.idle().await;
        }
    };
}
