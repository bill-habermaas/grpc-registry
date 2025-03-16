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

// Specific protobuf instance for a named group
#[derive(Debug)]
pub struct Service {
    pub url: String,    // gRPC service URL (host:port)
    pub stk: Option<String>,    // Server token of registree
    pub ctr: i32,       // number of keepalives
}

// Specific protobuf group basis
#[derive(Debug)]
pub struct Protobuf {
    pub name: String,   // protobuf name
    pub cltk: Option<String>,   // Client token for authorize
    pub services: Mutex<Vec<Mutex<Service>>>,
}

// General root for all protobuf grouping. It also provides data
// used throughout the registry application.
#[derive(Debug)]
pub struct Protobufs {
    pub keypair: RS256KeyPair,    // PEM keypair used to generatw JWTs
    pub protomap: HashMap<String, Mutex<Protobuf>>,
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
        &self, request: Request<registry::AuthorizeRequest>, )
        -> Result<Response<registry::AuthorizeResponse>, Status> {
        let req = request.into_inner();
        let response = authorize::handle_authorize(req.protobuf_name);
        Ok(Response::new(response))
    }

    async fn regs(
        &self, request: Request<registry::RegisterRequest>, )
        -> Result<Response<registry::RegisterResponse>, Status> {
        let req = request.into_inner();
        let response = registrations::handle_register(&req);
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

