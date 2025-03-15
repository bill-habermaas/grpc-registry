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
use crate::registry::{AuthorizeResponse};

// Authorize is used by clients to obtain a JWT allowing FIND requests against a specific
// protobuf service.
pub fn handle_authorize(protobuf_name: String) -> AuthorizeResponse {

    let protobufs = GDATA.get().unwrap().lock().unwrap();
    // lookup protobuf and error if not found
    let name = protobuf_name.clone();
    let protodef = common::find_protobuf(&protobufs, protobuf_name.clone());
    if protodef.is_some() {
        let kp = protobufs.keypair.clone();
        match jwt::create_token(kp, "client".to_string(), name.to_string(),
                                false,
                                Duration::from_hours(6)) {
            Ok(jwttoken) => {
                let jwt = jwttoken.clone();
                let mut p = protodef.unwrap().lock().unwrap();
                p.cltk = Some(jwt);
                let response = registry::AuthorizeResponse {
                    token: jwttoken,
                    status: None,
                };
                return response;
            },
            Err(e) => {
                let s = common::make_status_packet(common::StatusEnum::BADTOKEN, e);
                let response = registry::AuthorizeResponse {
                    token: "".to_string(),
                    status: Some(s),
                };
                return response
            },
        }
    } else {
        let s = common::make_status_packet(common::StatusEnum::NOTFOUND,
                                           "no matching protobuf definition".to_string());
        let response = registry::AuthorizeResponse {
            token: "".to_string(),
            status: Some(s)
        };
        return response;
    }
}
