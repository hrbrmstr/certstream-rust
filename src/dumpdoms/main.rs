// usage e.g. dumpdoms --dbpath=~/Data/cert_doms.1

use clap::{Parser};
use shellexpand;

use rocksdb::{DB, Options, IteratorMode}; // our database

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {

  // RocksDB path
  #[clap(short, long)]
  dbpath: String
  
}

fn main() {
  
  unsafe {
    libc::signal(libc::SIGPIPE, libc::SIG_DFL);
  }

  let args = Args::parse();

  let path = shellexpand::full(&args.dbpath).unwrap().to_string();

  let mut options = Options::default();
  options.set_error_if_exists(false);
  options.create_if_missing(false);
  
  let db = DB::open(&options, path.to_owned()).unwrap();

  let iter = db.iterator(IteratorMode::Start); // Always iterates forward
  for (key, _) in iter {
    println!("{}", String::from_utf8(key.to_vec()).unwrap());
  }

}