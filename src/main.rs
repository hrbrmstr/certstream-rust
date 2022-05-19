use std::process;
use std::str;
use std::{thread, time};

use itertools::Itertools; // for iterating over the array of domain strings

use url::{Url}; // tokio* uses real URLs
use tokio::io::{Result}; 
use tokio_tungstenite::{connect_async}; 
use futures_util::{StreamExt}; 

use rocksdb::{DB, Options}; // our database

// JSON record types for CertStream messages
mod json_types;

const CERTSTREAM_URL: &'static str = "wss://certstream.calidog.io/";
const ROCKSDB_PATH: &'static str = "/Users/hrbrmstr/Data/cert_doms.1";
const WAIT_AFTER_DISCONNECT: u64 =  5; // seconds

// in the deserialization part, the type of the returnd, parsed JSON gets wonky
macro_rules! assert_types {
  ($($var:ident : $ty:ty),*) => { $(let _: & $ty = & $var;)* }
}

// going all async b/c I may turn this into a library
#[tokio::main]
async fn main() -> Result<()> {
  
  // in case I want to do something on ^C
  ctrlc::set_handler(move || {
    process::exit(0x0000);
  }).expect("Error setting Ctrl-C handler");
    
  // IT TURNS OUT THE SERVER DOES DROP CONNECTIONS AND IT WASN'T JUST MY BAD CODE
  // NEEDED TO PUT THIS IN A LOOP W/SLEEPER CODE
  
  loop {
    
    // Setup RocksDB for writing
    let mut options = Options::default();
    options.set_error_if_exists(false);
    options.create_if_missing(true);
    options.create_missing_column_families(true);
    
    let db = DB::open(&options, ROCKSDB_PATH).unwrap();
    
    let certstream_url = Url::parse(CERTSTREAM_URL).unwrap(); // we need an actual Url type
    
    // connect to CertStream's encrypted websocket interface 
    let (wss_stream, _response) = connect_async(certstream_url).await.expect("Failed to connect");
    
    // the WebSocketStrem has sink/stream (read/srite) components; this is how we get to them
    let (mut _write, read) = wss_stream.split();
    
    // process messages as they come in
    let read_future = read.for_each(|message| async {
      
      match message {
        
        Ok(msg) => { // get the bytes as a str

          if let Ok(json_data) = msg.to_text() { // did the bytes convert to text ok?

            if json_data.len() > 0 { // do we actually have semi-valid JSON?

              match serde_json::from_str(&json_data) { // if this works
                
                Ok(record) => { // then derserialize JSON
                  
                  assert_types! { record: json_types::CertStream } // didn't need this before async ops
                  
                  // not all CertStream responses have the same JSON schema, so we have to do this
                  // tis possible i don't know a more idiomatic Rust way of doing this
                  
                  if let Some(data) = record.data {
                    if let Some(leaf) = data.leaf_cert {
                      if let Some(doms) = leaf.all_domains {
                        for dom in doms.into_iter().unique() {
                          // println!("{}", dom); // debugging
                          db.put(dom.to_ascii_lowercase(), "").unwrap(); // CertStream doms shld already be lowercase but making it explicit
                        }
                      }
                    }
                  }           
                }
                
                Err(err) => { println!("ERROR: {}", err) }
                
              }

            }

          }
          
        }
        
        Err(err) => { println!("ERROR: {}", err) }
        
      }
      
    });
    
    read_future.await;
    
    println!("Server disconnected…waiting {} seconds and retrying…", WAIT_AFTER_DISCONNECT);

    // kill the DB object and re-open since it's a fast operation
    let _ = DB::destroy(&Options::default(), ROCKSDB_PATH);

    thread::sleep(time::Duration::from_secs(WAIT_AFTER_DISCONNECT));
    
  }
  
}