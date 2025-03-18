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

// This client is a test-bed for the gRPC registry service. It exercises server
// functions and provides a working example of invoking a service through gRPC
// requests.


use std::collections::HashMap;
use registry:: registry_client::RegistryClient;

pub mod registry {
    tonic::include_proto!("registry");
}

#[tokio::test]
async fn test_authorize_for_unknown_protobuf() {
   let mut client = grpc_connect().await;

    let request = tonic::Request::new(registry::AuthorizeRequest {
        protobuf_name: "unknown-testproto".to_string(),
    });
    let response = client.auth(request).await;
    let a = response.unwrap();
    let b = a.into_inner();
    let c = b.status;
    let g = c.clone();
    let d = c.unwrap().code;
    println!("{:?}", g);
    assert_eq!(d, 1, "not protobuf does not exist: error-code={}", d);
}

#[tokio::test]
async fn test_register_service_functionality() {
    // 1 - Register a service
    // 2 - Issue a find which should return registered service.
    // 3 - Perform a keep-alive
    // 3 - DeRegister the service
    // 4 - Perform report.
    let mut client = grpc_connect().await;

    // define the protobuf service.
    let request = tonic::Request::new(registry::RegisterRequest {
        protobuf_name: "testproto".to_string(),
        protobuf_url: "localhost:8089".to_string(),
    });
    let response = client.regs(request).await;
    let a = response.unwrap();
    let b = a.into_inner();
    let token = b.token;
    let d = b.status;
    if d.is_some() {
        let g = d.clone();
        let h = g.clone();
        let e = d.unwrap().code;
        if h.is_some() {
            println!("{:?}", g);
            assert_eq!(e, 5, "register protobuf returned something unexpected");
        }
    }

    // Issue a find to get the service registration.else
    let tok1 = token.clone();
    let request = tonic::Request::new(registry::FindProviderRequest {
        registry_token: tok1,
        protobuf_name: "testproto".to_string(),
        by_round_robin: false,
        by_lowest_use: false,
    });
    let response = client.find(request).await;
    let a = response.unwrap();
    let b = a.into_inner();
    let d = b.service_url;
    assert_eq!(d, "localhost:8089".to_string(), "wrong service address");

    // Issue keep alive
    let tok2 = token.clone();
    let request = tonic::Request::new(registry::KeepaliveReport {
        token: tok2,
        number_requests: 0,
    });
    let response = client.alive(request).await;
    let a = response.unwrap();
    let b = a.into_inner();
    let c = b.status;
    assert!(c.is_none(), "not success");

    // Remove the registration.
    let tok3 = token.clone();
    let request = tonic::Request::new(registry::DeRegisterRequest {
        token: tok3,
    });
    let response = client.unreg(request).await;
    let a = response.unwrap();
    let b = a.into_inner();
    let c = b.status;
    assert!(c.is_none(), "not success");

    let tok4 = token.clone();
    let request = tonic::Request::new(registry::ProviderReportRequest {
        token: tok4,
    });
    let response = client.report(request).await;
    let a = response.unwrap();
    // print report -- instances for testproto should be empty
    println!("{:?}", a);
}

// Setup the gRPC connection for performing tests.
// Returns the gRPC client instance.
pub async fn grpc_connect() -> RegistryClient<Channel> {
    let params = getconfig();
    let server_addr = params.get("server_address").unwrap();
    let server_http = format!("http://{}", server_addr);
    //println!("Connecting to gRPC Server at {}", server_http);
    match RegistryClient::connect(server_http).await {
        Ok(c) => { return c; },
        Err(e) => {
            println!("connection error, tests cancelled: {}", e.to_string());
            std::process::exit(999);
        },
    };
}

// Load configuration parameters
use config::{Config};
use tonic::transport::Channel;
//use crate::registry::{DeRegisterRequest, ProviderReportRequest, ProviderReportResponse};

// Returns a hashmap of configuration parameters. The configuration file is
// shared between the client and server so the same network address is used
// in both.
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