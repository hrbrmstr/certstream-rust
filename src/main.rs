extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

use std::process;
use websocket::{ClientBuilder, OwnedMessage};
use itertools::Itertools;
use redis::Commands;

const CERTSTREAM: &'static str = "wss://certstream.calidog.io/";
const REDIS: &'static str = "redis://tycho.local/";
const DOMAIN_SET: &'static str = "cert_doms";

#[derive(Serialize, Deserialize)]
pub struct CertStream {
  pub data: Option<Data>,
  pub message_type: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct Data {
  pub cert_index: Option<i64>,
  pub cert_link: Option<String>,
  pub leaf_cert: Option<LeafCert>,
  pub seen: Option<f64>,
  pub source: Option<Source>,
  pub update_type: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct LeafCert {
  pub all_domains: Option<Vec<String>>,
  pub extensions: Option<Extensions>,
  pub fingerprint: Option<String>,
  pub issuer: Option<Issuer>,
  pub not_after: Option<i64>,
  pub not_before: Option<i64>,
  pub serial_number: Option<String>,
  pub signature_algorithm: Option<String>,
  pub subject: Option<Issuer>,
}

#[derive(Serialize, Deserialize)]
pub struct Extensions {
  #[serde(rename = "authorityInfoAccess")]
  pub authority_info_access: Option<String>,
  #[serde(rename = "authorityKeyIdentifier")]
  pub authority_key_identifier: Option<String>,
  #[serde(rename = "basicConstraints")]
  pub basic_constraints: Option<String>,
  #[serde(rename = "certificatePolicies")]
  pub certificate_policies: Option<String>,
  #[serde(rename = "ctlSignedCertificateTimestamp")]
  pub ctl_signed_certificate_timestamp: Option<String>,
  #[serde(rename = "extendedKeyUsage")]
  pub extended_key_usage: Option<String>,
  #[serde(rename = "keyUsage")]
  pub key_usage: Option<String>,
  #[serde(rename = "subjectAltName")]
  pub subject_alt_name: Option<String>,
  #[serde(rename = "subjectKeyIdentifier")]
  pub subject_key_identifier: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct Issuer {
  #[serde(rename = "C")]
  pub c: Option<String>,
  #[serde(rename = "CN")]
  pub cn: Option<String>,
  #[serde(rename = "L")]
  pub l: Option<serde_json::Value>,
  #[serde(rename = "O")]
  pub o: Option<String>,
  #[serde(rename = "OU")]
  pub ou: Option<serde_json::Value>,
  #[serde(rename = "ST")]
  pub st: Option<serde_json::Value>,
  pub aggregated: Option<String>,
  #[serde(rename = "emailAddress")]
  pub email_address: Option<serde_json::Value>,
}

#[derive(Serialize, Deserialize)]
pub struct Source {
  pub name: Option<String>,
  pub url: Option<String>,
}

fn main() {

  ctrlc::set_handler(move || {
    process::exit(0x0000);
  }).expect("Error setting Ctrl-C handler");

  let mut redis_conn = redis::Client::open(REDIS)
                        .expect("Invalid connection URL")
                        .get_connection()
                        .expect("Failed to connect to Redis");
      
  let mut client = ClientBuilder::new(CERTSTREAM)
                    .unwrap()
                    .connect_secure(None)
                    .unwrap();
                    
  for msg in client.incoming_messages() {

    match msg {

      Ok(msg) => {

        if let OwnedMessage::Text(s) = msg { 
          
          let model: CertStream = serde_json::from_str(&s).unwrap();
          
          if let Some(data) = model.data {
            if let Some(leaf) = data.leaf_cert {
              if let Some(doms) = leaf.all_domains {
                for dom in doms.into_iter().unique() {
                  let _: ()  = redis_conn
                                .sadd(DOMAIN_SET, dom.to_lowercase().to_string())
                                .expect("failed to execute SADD for 'cert_doms'");
                }
              }
            }
          } 
        }
      },
      Err(_err) => { break }
    }

  }    
}