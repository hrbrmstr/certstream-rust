// usage e.g. certstream --dbpath=~/Data/cert_doms.1

use std::process;
use std::str;
use std::{thread, time};

use clap::{Parser};
use shellexpand;

use itertools::{Itertools}; // for iterating over the array of domain strings

use url::{Url}; // tokio* uses real URLs
use tokio::io::{Result}; 
use tokio_tungstenite::{connect_async}; 
use futures_util::{StreamExt}; 

use rocksdb::{DB, Options}; // our database

// JSON record types for CertStream messages
mod json_types;

const CERTSTREAM_URL: &'static str = "wss://certstream.calidog.io/";
const WAIT_AFTER_DISCONNECT: u64 =  5; // seconds

// in the deserialization part, the type of the returnd, parsed JSON gets wonky
macro_rules! assert_types {
  ($($var:ident : $ty:ty),*) => { $(let _: & $ty = & $var;)* }
}

// Extract all domains from a CertStream-compatible CTL websockets server to RocksDB
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {

  // RocksDB path
  #[clap(short, long)]
  dbpath: String,

  // server to use (defaults to CertStream's)
  #[clap(short, long, default_value_t = String::from(CERTSTREAM_URL))]
  server: String,

  // how long to wait to connect after a remote disconnect
  #[clap(short, long, default_value_t = WAIT_AFTER_DISCONNECT)]
  patience: u64
  
}

// going all async b/c I may turn this into a library
#[tokio::main]
async fn main() -> Result<()> {

  let args = Args::parse();

  let path = shellexpand::full(&args.dbpath).unwrap().to_string();

  // may want todo someting on ^C
  ctrlc::set_handler(move || {
    process::exit(0x0000);
  }).expect("Error setting Ctrl-C handler");

  loop { // server is likely to drop connections
    
    // Setup RocksDB for writing
    let mut options = Options::default();
    options.set_error_if_exists(false);
    options.create_if_missing(true);
    options.create_missing_column_families(true);
    
    let db = DB::open(&options, path.to_owned()).unwrap();
    
    let certstream_url = Url::parse(args.server.as_str()).unwrap(); // we need an actual Url type
    
    // connect to CertStream's encrypted websocket interface 
    let (wss_stream, _response) = connect_async(certstream_url).await.expect("Failed to connect");
    
    // the WebSocketStrem has sink/stream (read/srite) components; this is how we get to them
    let (mut _write, read) = wss_stream.split();
    
    // process messages as they come in
    let read_future = read.for_each(|message| async {
      match message {

        Ok(msg) => { // we have the websockets message bytes as a str
           
          if let Ok(json_data) = msg.to_text() { // did the bytes convert to text ok?
            if json_data.len() > 0 { // do we actually have semi-valid JSON?
              match serde_json::from_str(&json_data) { // if deserialization works
                Ok(record) => { // then derserialize JSON
                  
                  assert_types! { record: json_types::CertStream } 
                  
                  for dom in record.data.leaf_cert.all_domains.into_iter().unique() {
                    // CertStream doms shld already be lowercase but making it explicit
                    db.put(dom.to_ascii_lowercase(), "").unwrap(); 
                  }
                                                                
                }
                
                Err(err) => { eprintln!("{}", err) }
                
              }
            }
          }
        }
        
        Err(err) => { eprintln!("{}", err) }
        
      }
      
    });
    
    read_future.await;
    
    eprintln!("Server disconnected…waiting {} seconds and retrying…", args.patience);

    // kill the DB object and re-open since it's a fast operation
    let _ = DB::destroy(&Options::default(), path.to_owned());

    // wait for a bit to be kind to the server
    thread::sleep(time::Duration::from_secs(args.patience));
    
  }
  
}