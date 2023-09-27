mod config;
mod contents;
mod opt;
mod trial;

use config::Config;
use opt::Opt;

#[derive(Debug)]
pub struct Env {
    pub config: Config,
    pub opt: Opt,
}

fn main() {
    let opt = <Opt as clap::Parser>::parse();
    if opt.debug {
        dbg!(&opt);
    }

    let config = config::load(&opt);
    if opt.debug {
        dbg!(&config);
    }

    let env = Env { config, opt };

    let mut contents = contents::generate(&env);

    for line in &mut *contents {
        println!("{}", line);
    }
    println!(":: restart ::");
    contents.restart();
    for line in &mut *contents {
        println!("{}", line);
    }
}
