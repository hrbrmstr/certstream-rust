use std::process;
use std::str;

use itertools::Itertools;

use tokio::io::{Result}; // AsyncWriteExt, Result};
use tokio_tungstenite::{connect_async}; //, tungstenite::protocol::Message};
use futures_util::{StreamExt}; // , SinkExt};

use url::{Url};
use rocksdb::{DB, Options};

// JSON record types for CertStream messages
mod json_types;

const CERTSTREAM_URL: &'static str = "wss://certstream.calidog.io/";
const ROCKSDB_PATH: &'static str = "/Users/hrbrmstr/Data/cert_doms.1";

macro_rules! assert_types {
  ($($var:ident : $ty:ty),*) => { $(let _: & $ty = & $var;)* }
}

#[tokio::main]
async fn main() -> Result<()> {
  
  ctrlc::set_handler(move || {
    process::exit(0x0000);
  }).expect("Error setting Ctrl-C handler");
  
  // Setup RocksDB for writing
  let mut options = Options::default();
  options.set_error_if_exists(false);
  options.create_if_missing(true);
  options.create_missing_column_families(true);
  
  let db = DB::open(&options, ROCKSDB_PATH).unwrap();
  
  let certstream_url = Url::parse(CERTSTREAM_URL).unwrap(); // we need an actual Url type
  
  // IT TURNS OUT THE SERVER DOES DROP CONNECTIONS AND IT WASN'T JUST MY BAD CODE
  // NEED TO PUT THIS IN A LOOP W/SLEEPER CODE

  // connect to CertStream's encrypted websocket interface 
  let (wss_stream, _response) = connect_async(certstream_url).await.expect("Failed to connect");
  
  // the WebSocketStrem has sink/stream (read/srite) components; this is how we get to them
  let (mut _write, read) = wss_stream.split();
  
  // process messages as they come in
  let read_future = read.for_each(|message| async {
    
    match message {
      
      Ok(msg) => { // get the bytes as a str
        if let Ok(json_data) = msg.to_text() {
          if json_data.len() > 0 { 
            match serde_json::from_str(&json_data) { // derserialize JSON
              
              Ok(record) => {
                
                assert_types! { record: json_types::CertStream }
                
                // not all CertStream responses have the same JSON schema, so we have to do this
                // tis possible i don't know a more idiomatic Rust way of doing this
                
                if let Some(data) = record.data {
                  if let Some(leaf) = data.leaf_cert {
                    if let Some(doms) = leaf.all_domains {
                      for dom in doms.into_iter().unique() {
                        // println!("{}", dom);
                        db.put(dom, "").unwrap();
                      }
                    }
                  }
                }           
              }
              
              Err(err) => {
                println!("ERROR: {}", err)
              }
              
            }
          }
        }
        
      }
      
      Err(err) => {
        println!("ERROR: {}", err)
      }
      
    }
    
  });
  
  read_future.await;
  
  Ok(())
  
}