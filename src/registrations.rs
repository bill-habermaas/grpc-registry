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

use crate::{common, jwt, registry, GDATA};
use crate::registry::{RegisterResponse};

pub fn handle_register(req: &registry::RegisterRequest) -> registry::RegisterResponse {

    let s = common::make_status_packet(common::StatusEnum::SERVERROR, "not supported".to_string());
    let r = registry::RegisterResponse{
        token: "".to_string(),
        status: Some(s),
    };
    r
}