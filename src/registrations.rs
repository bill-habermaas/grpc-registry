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

#[derive(Debug)]
struct Service {
    url: String,
    token: String,
}

#[derive(Debug)]
pub struct Protobuf {
    name: String,
    services: Vec<Service>,
}

#[derive(Debug)]
pub struct Protobufs {
    protobuf_map: HashMap<String, Protobuf>,
}
impl Protobufs {
    fn new() -> Protobufs {
        Protobufs {
            protobuf_map: HashMap::new(),
        }
    }
    fn add_protobuf(&mut self, key: &str, protobuf: Protobuf) {
        self.protobuf_map.insert(key.to_string(), protobuf);
    }
}

pub fn registration_init() -> Protobufs {
    let rootnode = Protobufs::new();
    rootnode
}

pub fn create_protobuf(protobuf_name: String, mut root: Protobufs) {
    let protobuf = Protobuf{
        name: protobuf_name.clone(),
        services: Vec::new(),
    };
    root.add_protobuf(&*protobuf_name, protobuf);
}




