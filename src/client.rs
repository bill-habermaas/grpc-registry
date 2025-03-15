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

use registry:: registry_client::RegistryClient;


pub mod registry {
    tonic::include_proto!("registry");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = RegistryClient::connect("http://[::1]:50055").await?;

    let request = tonic::Request::new(registry::AuthorizeRequest {
        protobuf_name: "testproto".to_string(),
    });

    println!("Sending request to gRPC Server...");
    let response = client.auth(request).await?;

    println!("RESPONSE={:?}", response);

    Ok(())
}