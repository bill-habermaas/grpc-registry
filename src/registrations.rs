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
use jwt_simple::prelude::Duration;
use crate::{common, jwt, registry, GDATA};
use crate::common::make_service;
use crate::registry::{DeRegisterRequest, DeRegisterResponse, FindProviderRequest, FindProviderResponse, KeepAliveResponse, KeepaliveReport};

// Handle protobuf registration.
pub fn handle_register(req: &registry::RegisterRequest) -> registry::RegisterResponse {

    let name = req.protobuf_name.to_string();
    let name2 = name.clone();
    let name3 = name2.clone();
    let name4 = name3.clone();
    let url = req.protobuf_url.to_string();
    let url1 = url.clone();

    let mut protobufs = GDATA.get().unwrap().lock().unwrap();
    let protobuf = common::find_protobuf(&protobufs, name);
    if protobuf.is_none() {
        // protobuf does not exist.
        let r = common::add_protobuf(&mut protobufs, name3);
        if r.is_err() {
            let s = common::make_status_packet(common::StatusEnum::SERVERROR, r.unwrap_err());
            let rsp = registry::RegisterResponse {
                token: "".to_string(),
                status: Some(s),
            };
            return rsp;
        }
    }
    // Refetch protobuf incase it didn't exist and was just created.
    let tmpprot = common::find_protobuf(&protobufs, name2);
    let service = make_service(url);
    let mut protobuf = tmpprot.unwrap().lock().unwrap();
    let r = common::add_service(&mut protobuf, service, url1);
    if r.is_err() {
        let s = common::make_status_packet(common::StatusEnum::SERVERROR, r.unwrap_err());
        let r = registry::RegisterResponse {
            token: "".to_string(),
            status: Some(s),
        };
        return r;
    }

    // Create the token for the response
    let kp = protobufs.keypair.clone();
    match jwt::create_token(kp, "server".to_string(), name4.to_string(), false,
                                Duration::from_hours(6)) {
        Ok(jwttoken) => {
            let _jwt = jwttoken.clone();
            //let mut svc = service.unwrap().lock().unwrap();
            //let svc.stk = Some(jwt);

            let response = registry::RegisterResponse {
                token: jwttoken,
                status: None,
            };
            return response;
        },
        Err(e) => {
            let s = common::make_status_packet(common::StatusEnum::BADTOKEN, e);
            let response = registry::RegisterResponse {
                token: "".to_string(),
                status: Some(s),
            };
            return response
        },
    }
}

pub fn handle_deregister(_req: DeRegisterRequest) -> DeRegisterResponse {
    let rsp = DeRegisterResponse {
        status: None,
    };
    rsp
}

pub fn handle_find_provider(_req: FindProviderRequest) -> FindProviderResponse {
    let rsp = FindProviderResponse {
        registry_token: "".to_string(),
        service_url: "".to_string(),
        status: None,
    };
    rsp
}

pub fn handle_keep_alive(_req: KeepaliveReport) -> KeepAliveResponse {
    let rsp = KeepAliveResponse {
        status: None,
    };
    rsp
}
