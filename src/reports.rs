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
use crate::common::{get_keypair, make_status_packet};
use crate::{common, jwt, GDATA};
use crate::registry::{ByProvider, ByProviderInstance, ProviderReportRequest, ProviderReportResponse};

pub fn handle_provider_report(req: ProviderReportRequest) -> ProviderReportResponse {
    let token = req.token;
    let kp = get_keypair();
    let claim = jwt::validate_token(kp.unwrap(), token);
    if claim.is_err() {
        let s = make_status_packet(common::StatusEnum::AUTHERROR, claim.unwrap_err());
        let response = ProviderReportResponse {
            providers: Vec::new(),
            status: Some(s),
        };
        return response;
    }

    let mut byproviders = Vec::new();
    let protobufs= GDATA.get().unwrap().lock().unwrap();

    //let providers = protobufs.protomap.values().cloned().collect::<Vec<Protobuf>>();
    for provider in protobufs.protomap.values() {
        let protoitem = provider.lock().unwrap();
        let protoname = protoitem.name.clone();
        let mut byproto = ByProvider {
            protobuf_name: protoname,
            instances: Vec::new(),
        };
        // iterate through services
        for i in 0..protoitem.services.len() {
            let lsrv = &protoitem.services[i];
            let srv = lsrv.lock().unwrap();
            let url = srv.url.clone();
            let serv = ByProviderInstance {
                service_url: url,
                requests: srv.ctr,
            };
            byproto.instances.push(serv);
        }
        byproviders.push(byproto);
    }

    let rsp = ProviderReportResponse {
        providers: byproviders,
        status: None,
    };
    rsp
}
