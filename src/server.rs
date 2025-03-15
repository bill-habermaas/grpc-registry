/*
 * Copyright 2025 Habermaas Systems, Inc. All rights reserved.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *     *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 *
 */

use tonic::{transport::Server, Request, Response, Status};

pub mod jwt;
pub mod common;
pub mod authorize;
pub mod registrations;

use registry::registry_server::{Registry, RegistryServer};


// Import the generated proto-rust file into a module
pub mod registry {
    tonic::include_proto!("registry");
}

use std::sync::Mutex;
use std::collections::HashMap;
use std::string::ToString;

// Specific protobuf instance for a named group
#[derive(Debug)]
struct Service {
    url: String,
    ctr: i32,
}

// Specific protobuf group basis
#[derive(Debug)]
struct Protobuf {
    name: String,
    services: Mutex<Vec<Mutex<Service>>>,
}

// General root for all protobuf grouping. It also provides data
// used throughout the registry application.
#[derive(Debug)]
struct Protobufs {
    keypair: RS256KeyPair,    // PEM keypair used to generatw JWTs
    protomap: HashMap<String, Mutex<Protobuf>>,
}

use once_cell::sync::OnceCell;
static GDATA: OnceCell<Mutex<Protobufs>> = OnceCell::new();

// Implement the service skeleton for the "registry" service
// defined in the proto
#[derive(Debug, Default)]
pub struct MyRegistry {
}

// Implement the service function(s) defined in the proto
// for the registry service
#[tonic::async_trait]
impl Registry for MyRegistry {

    async fn auth(
        &self,
        request: Request<registry::AuthorizeRequest>,
    ) -> Result<Response<registry::AuthorizeResponse>, Status> {
        let req = request.into_inner();
        let response = unsafe { handle_authorize(req.protobuf_name) };
        Ok(Response::new(response))
    }
}

// Runtime to run our server
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    let params = getconfig();
    let keyfile = params.get("public_key_file").unwrap().clone();
    let keyname = keyfile.clone();
    match jwt::load_pem(keyfile) {
        Ok(kp) => {
            let ps = Protobufs {
                keypair: kp,
                protomap: HashMap::new(),
            };
            let _ = GDATA.set(Mutex::new(ps));
        },
        Err(e) => {
            println!("Failed to load key pair from {} {}", keyname, e);
            std::process::exit(97);
        },
    }

    let addr = "[::1]:50055".parse()?;
    let serv = MyRegistry::default();

    println!("Starting gRPC Registration server...");
    Server::builder()
        .add_service(RegistryServer::new(serv))
        .serve(addr)
        .await?;

    Ok(())
}

// Load configuration parameters
use config::{Config};
use jwt_simple::algorithms::RS256KeyPair;
use authorize::handle_authorize;

pub fn getconfig() -> HashMap<String, String> {
    let settings = Config::builder()
        // Add in `./Settings.toml`
        .add_source(config::File::with_name("setting.toml"))
        // Add in settings from the environment (with a prefix of APP)
        // Eg.. `APP_DEBUG=1 ./target/app` would set the `debug` key
        .add_source(config::Environment::with_prefix("APP"))
        .build()
        .unwrap();

    let cfg = settings.clone();
    let themap: HashMap<String, String> =
        cfg.try_deserialize::<HashMap<String, String>>()
            .unwrap();
    themap
}

///////////////////////////////////////////////////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////////////////
//// Caveats - Mutex objects are used to enforce concurrent safety. The
//// rational for these helper routines is that the mutex is in a locked
//// state before being called. This ensures higher level functions will
//// release locks based upon scoping.
///////////////////////////////////////////////////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////////////////

pub fn make_service(url: String) -> Mutex<Service> {
    let s = Service{
        url: url,
        ctr: 0,
    };
    let m = Mutex::new(s);
    m
}

// Make a new protobuf group definition
pub fn make_protobuf(protobuf_name: &String) -> Mutex<Protobuf> {
    let p = Protobuf {
        name: protobuf_name.to_string(),
        services: Vec::new().into(),
    };
    let m = Mutex::new(p);
    m
}

// This must be called when protobufs structure has been locked. Locking is not done in called routines.
pub fn find_protobuf(protobufs: &Protobufs, protobuf_name: String) -> Option<&Mutex<Protobuf>> {
    let p = protobufs.protomap.get(&protobuf_name);
    if p.is_some() {
        return p
    }
    None
}

// Add a new protobuf group to the collection of all groups
pub fn add_protobuf(protobufs: &mut Protobufs, protobuf_name: String) -> Result<(), String> {
    let name = protobuf_name.clone();
    let name2 = protobuf_name.clone();
    if ( find_protobuf(protobufs, protobuf_name)) .is_some() {
        return Err("Protobuf is already registered".to_string())
    }
    let p = make_protobuf(&name);
    protobufs.protomap.insert(name2, p);
    Ok(())
}

// Add a new service definition to a protobuf grouping.
pub fn add_service(protobuf: &mut Protobuf, url: String) -> Result<(), String> {
    //if false { //check_for_dup_urls(&protobuf.services, url) == true {
    //Err(_s);
    //} else
    {
        let s = make_service(url);
        //let aa = protobuf.unwrap();
        //let bb = aa.lock().unwrap();
        let mut cc = protobuf.services.lock().unwrap();
        cc.push(s);
    }
    Ok(())
}

// Check for duplicate protobuf urls in the protobuf group.
fn check_for_dup_urls(ids: &Vec<Mutex<Service>>, url: String) -> bool {
    if ids.len() > 0 {
        for x in 0..ids.len() {
            let s = ids[x].lock().unwrap();
            if url == s.url {
                return true;
            }
        }
    }
    return false;
}
