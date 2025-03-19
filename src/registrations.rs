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
use crate::{common, jwt, registry, Protobuf, GDATA};
use crate::common::{find_protobuf, get_keypair, make_status_packet};
use crate::registry::{DeRegisterRequest, DeRegisterResponse, FindProviderRequest, FindProviderResponse, KeepAliveResponse, KeepaliveReport};

// Handle protobuf registration. The protobuf name ans service url are encoded in
// the jwt token so token validation can provide this information when deregistering.
pub fn handle_register(req: &registry::RegisterRequest) -> registry::RegisterResponse {

    let name1 = req.protobuf_name.to_string();
    let name2 = name1.clone();
    let name3 = name2.clone();
    let url1 = req.protobuf_url.to_string();
    let url2 = url1.clone();

    // username contains the url
    // subject contains the protobuf name.
    let token = common::make_token(url2.to_string(), name2.to_string(),
                                   false, Duration::from_hours(12));

    let mut protobufs = GDATA.get().unwrap().lock().unwrap();
    let protobuf = common::find_protobuf(&protobufs, name1);
    if protobuf.is_none() {
        // protobuf does not exist.
        let r = common::add_protobuf(&mut protobufs, name2);
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
    let tmpprot = common::find_protobuf(&protobufs, name3);
    let mut protobuf = tmpprot.unwrap().lock().unwrap();
    let token3 = token.clone();
    let r = common::add_service(&mut protobuf, url2, token);
    if r.is_err() {
        let s = common::make_status_packet(common::StatusEnum::SERVERROR, r.unwrap_err());
        let r = registry::RegisterResponse {
            token: "".to_string(),
            status: Some(s),
        };
        return r;
    }
    let rsp = registry::RegisterResponse {
        token: token3.unwrap(),
        status: None,
    };
    rsp
}

// Remove a protocol registration
pub fn handle_deregister(req: DeRegisterRequest) -> DeRegisterResponse {
    let (key, response) = inner_deregister(req);
    if key.len() > 0  {
        let mut protobufs = GDATA.get().unwrap().lock().unwrap();
        protobufs.protomap.remove(&key);
    }
    response
}
// The inner deregister does the bulk of the work. But the handle_reregister is the
// hook to remove the map entry for the protobuf from the hashmap when all services
// have been removed. This is necessary so the protobufs instance is handled out
// of scope from the mainline processing.
fn inner_deregister(req: DeRegisterRequest) -> (String, DeRegisterResponse) {
    let token = req.token;
    let kp = get_keypair();
    let claim = jwt::validate_token(kp.unwrap(), token);
    if claim.is_err() {
        let s = make_status_packet(common::StatusEnum::AUTHERROR, claim.unwrap_err());
        let response = DeRegisterResponse {
            status: Some(s),
        };
        return ("".to_string(), response);
    }
    let info = claim.unwrap();
    let proto_name = info.subject.unwrap();
    let url = info.custom.user_name;
    // find the protobuf containing the protobuf group
    let pname = proto_name.clone();
    let protobufs = GDATA.get().unwrap().lock().unwrap();
    let protocol = find_protobuf(&protobufs, proto_name);
    if protocol.is_none() {
        return unreg_not_found();
    }
    // search through the service index to find the registration.else
    let mut protobuf = protocol.unwrap().lock().unwrap();
    let rsp = DeRegisterResponse {
        status: None,
    };
    if find_entry_to_remove(&mut protobuf, url) == false {
        return unreg_not_found();
    } else {
        if protobuf.services.is_empty() {
            return (pname.to_string(), rsp);
        }
    }
    ("".to_string(), rsp)
}

// locate the proper service entry to remove.
fn find_entry_to_remove(protobuf: &mut Protobuf, url: String) -> bool {
    let ct = protobuf.services.len();
    if ct < 1 { return false; }
    let usize = search_services(protobuf, url);
    if usize == 0 { return false; }
    let usize = usize -1 ;
    // found the service entry, get id of it
    protobuf.services.remove(usize);
    return true;
}

fn search_services(protobuf: &mut Protobuf, url: String) -> usize {
    let ct = protobuf.services.len();
    for i in 0..ct {
        let m = &protobuf.services[i];
        let p = m.lock().unwrap();
        if p.url == url {
            return i + 1;
        }
    }
    return 0;
}

fn unreg_not_found() -> (String, DeRegisterResponse) {
    let s = make_status_packet(common::StatusEnum::NOTFOUND,
                               "protobuf not found".to_string());
    let response = DeRegisterResponse {
        status: Some(s),
    };
    return ("".to_string(), response);
}

// find the provider
pub fn handle_find_provider(req: FindProviderRequest) -> FindProviderResponse {
    let token = req.registry_token;
    let protobuf_name = req.protobuf_name;
    let kp = get_keypair();
    let claim = jwt::validate_token(kp.unwrap(), token);
    if claim.is_err() {
        let s = make_status_packet(common::StatusEnum::AUTHERROR, claim.unwrap_err());
        let response = FindProviderResponse {
            service_url: "".to_string(),
            status: Some(s),
        };
        return response;
    }
    // token is good. See if protobuf exists
    let protobufs = GDATA.get().unwrap().lock().unwrap();
    let r = find_protobuf(&protobufs, protobuf_name);
    if r.is_none() {
        // Doesn't exist
        let s = make_status_packet(common::StatusEnum::NOTFOUND,
                                   "protobuf does not exist".to_string());
        let response = FindProviderResponse {
            service_url: "".to_string(),
            status: Some(s),
        };
        return response;
    }
    let protobuf = r.unwrap().lock().unwrap();
    if protobuf.services.is_empty() {
        let s = make_status_packet(common::StatusEnum::NOTFOUND,
                                   "protobuf does not exist".to_string());
        let response = FindProviderResponse {
            service_url: "".to_string(),
            status: Some(s),
        };
        return response;
    }

    // Pick off the start of service list
    let s = protobuf.services.get(0);
    let svc = s.unwrap().lock().unwrap();
    let service_url = &svc.url;
    // load balancing code goes here
    let rsp = FindProviderResponse {
        service_url: service_url.to_string(),
        status: None,
    };
    rsp
}

// Handle keep alive request
pub fn handle_keep_alive(req: KeepaliveReport) -> KeepAliveResponse {
    let token = req.token;
    let count = req.number_requests;
    let kp = get_keypair();
    let claim = jwt::validate_token(kp.unwrap(), token);
    if claim.is_err() {
        let s = make_status_packet(common::StatusEnum::AUTHERROR, claim.unwrap_err());
        let response = KeepAliveResponse {
            status: Some(s),
        };
        return response;
    }
    // token is good
    let c = claim.unwrap();
    let protobuf_name = c.subject;
    let url = c.custom.user_name;
    // Now find the service item
    let protobufs = GDATA.get().unwrap().lock().unwrap();
    let protobuf = find_protobuf(&protobufs, protobuf_name.unwrap());
    if protobuf.is_none() {
        let s = make_status_packet(common::StatusEnum::NOTFOUND,
                                   "protobuf does not exist".to_string());
        let response = KeepAliveResponse {
            status: Some(s),
        };
        return response;
    }
    // lock the protobuf struct and find the matching service
    let p = protobuf.unwrap().lock().unwrap();
    if p.services.is_empty() {
        let s = make_status_packet(common::StatusEnum::NOTFOUND,
                                   "protobuf does not exist".to_string());
        let response = KeepAliveResponse {
            status: Some(s),
        };
        return response;
    }

    let ct = p.services.len();
    for i in 0..ct {
        let m = &p.services[i];
        let mut x = m.lock().unwrap();
        if x.url == url {
            x.ctr = count;
        }
    }
    let rsp = KeepAliveResponse {
        status: None,
    };
    rsp
}
