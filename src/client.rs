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
use std::collections::HashMap;
use registry:: registry_client::RegistryClient;


pub mod registry {
    tonic::include_proto!("registry");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    let params = getconfig();
    let server_addr = params.get("server_address").unwrap();
    let server_http = format!("http://{}", server_addr);
    println!("Connecting to gRPC Server at {}", server_http);
    let mut client = RegistryClient::connect(server_http).await?;

    let request = tonic::Request::new(registry::AuthorizeRequest {
        protobuf_name: "testproto".to_string(),
    });
    let response = client.auth(request).await?;

    println!("RESPONSE={:?}", response);

    Ok(())
}

// Load configuration parameters
use config::{Config};
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