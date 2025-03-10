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

use jwt_simple::algorithms::RS256KeyPair;
use jwt_simple::prelude::Duration;
use crate::{common, jwt, registry};
use crate::registry::{AuthorizeResponse};
use crate::registry::StatusCodes::{Badtoken, NotFound};

pub fn handle_authorize(protobuf_name: &String, key: RS256KeyPair) -> AuthorizeResponse {

    // lookup protobuf and error if not found
    let name = protobuf_name.clone();
    let protodef = common::findprotobuf(*protobuf_name);
    let token = None;
    let status = None;
    if protodef.is_some() {
        match jwt::create_token(key, "client".to_string(), name.to_string(),
                                false,
                                Duration::from_hours(6)) {
            Ok(mut jwttoken) => {
                let response = registry::AuthorizeResponse {
                    token: jwttoken,
                    status: None,
                };
                return response;
            },
            Err(e) => {
                let status = registry::StatusPacket {
                    code: Badtoken,
                    error_message: e.to_string(),
                };
                let response = registry::AuthorizeResponse {
                    token: None,
                    status: Some(status),
                };
                return response
            },
        }
    }
}