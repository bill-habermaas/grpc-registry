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

use std::sync::Mutex;
use jwt_simple::prelude::{Duration, RS256KeyPair};
use crate::{jwt, Protobuf, Protobufs, Service, KPAIR};
use crate::registry;

///////////////////////////////////////////////////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////////////////
//// Caveats - Mutex objects are used to enforce concurrent safety. The
//// rational for these helper routines is that the mutex is in a locked
//// state before being called. This ensures higher level functions will
//// release locks based upon scoping.
///////////////////////////////////////////////////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////////////////

pub fn make_token(user: String, subject: String, isadmin: bool, duration: Duration) -> Option<String> {
    let kpo = get_keypair();
    match jwt::create_token(kpo.unwrap(), user.to_string(), subject.to_string(), isadmin,
                            duration) {
                           // Duration::from_hours(12)) {
        Ok(jwttoken) => { return Some(jwttoken); },
        Err(e) => {
            println!("make token error: {}", e);
        },
    };
    return None;
}

// This is hack to make sure the keypair is set in all situations. When tests are run
// the normal server initialization does not happen so we need to check and make the
// keypair on the fly so the tests won't panic when it is None.
pub fn get_keypair() -> Option<&'static RS256KeyPair> {
    let kp = KPAIR.get();
    if kp.is_none() {
        let kpr = RS256KeyPair::generate(4096);
        let kp = kpr.unwrap();
        let _p = KPAIR.set(kp);
    }
    return KPAIR.get();
}

#[test]
fn test_make_token() {
    let fp = KPAIR.get();
    if fp.is_none() {

    }
    let t= make_token("test".to_string(),
    "subject".to_string(), false, Duration::from_mins(5));
    if t.is_none() {
        assert!(false);
    }
    else {
        println!("{}", t.unwrap());
    }
}

pub fn make_service(url: String, token: Option<String>) -> Mutex<Service> {
    let s = Service{
        url: url,
        stk: token,
        ctr: 0,
    };
    let m = Mutex::new(s);
    m
}

// Make a new protobuf group definition
pub fn make_protobuf(protobuf_name: &String) -> Mutex<Protobuf> {
    let p = Protobuf {
        name: protobuf_name.to_string(),
        cltk: None,
        services: Vec::new().into(),
    };
    let m = Mutex::new(p);
    m
}

// This must be called when protobufs structure has been locked. Locking is not done in called routines.
pub fn find_protobuf(protobufs: &Protobufs, protobuf_name: String) -> Option<&Mutex<Protobuf>> {
    let p = protobufs.protomap.get(&protobuf_name);
    if p.is_some() {
        return p
    }
    None
}

// Add a new protobuf group to the collection of all groups
pub fn add_protobuf(protobufs: &mut Protobufs, protobuf_name: String) -> Result<(), String> {
    let name = protobuf_name.clone();
    let name2 = protobuf_name.clone();
    if ( find_protobuf(protobufs, protobuf_name)) .is_some() {
        return Err("Protobuf is already registered".to_string())
    }
    let p = make_protobuf(&name);
    protobufs.protomap.insert(name2, p);
    Ok(())
}

#[test]
fn test_add_protobuf() {
    let mut protobufs = Protobufs{
        protomap: HashMap::new(),
    };

    let p = add_protobuf(&mut protobufs, "testproto".to_string());
    if p.is_ok() {
        let b = protobufs.protomap.contains_key(&"testproto".to_string());
        assert_eq!(b, true,"missing protobuf")
    } else {
        panic!("error returned");
    }
}

// Add a new service definition to a protobuf grouping.
pub fn add_service(protobuf: &mut Protobuf, url: String, token: Option<String>) -> Result<(), String> {
    let url1 = url.clone();
    if check_for_dup_urls(&protobuf, url) == true {
        let s = "duplicate url in service".to_string();
        return Err(s);
    } else {
        let s = make_service(url1, token);
        //let mut cc = &protobuf.services;
        protobuf.services.push(s);
    }
    Ok(())
}

#[test]
fn test_add_service_to_protobuf() {
    let mut protobuf = Protobuf {
        name: "proto1".to_string(),
        cltk: None,
        services: Vec::new(),
    };
    let r = add_service(&mut protobuf, "url1".to_string(), None);
    if r.is_ok() {
        let _ = make_service("url1".to_string(), None);
        // the following must fail because the service url was just created.
        let rr = add_service(&mut protobuf, "url1".to_string(), None);
        let err = rr.unwrap_err();
        assert_eq!(err, "duplicate url in service");
    }
}

// Check for duplicate protobuf urls in this protobuf group.
fn check_for_dup_urls(protobuf: &Protobuf, url: String) -> bool {
    let ct = protobuf.services.len();
    if ct < 1 { return false; }
    for i in 0..ct {
        let m = &protobuf.services[i];
        let p = m.lock().unwrap();
        if p.url == url { return true; }
    }
    return false;
}

#[test]
fn duplicate_in_service_list() {
    let mut protobuf = Protobuf {
        name: "proto1".to_string(),
        cltk: None,
        services: Vec::new(),
    };
    for i in 0..5 {
        let s = Service {
            url: format ! ("url-{}", i),
            stk: None,
            ctr: 0,
        };
        let m = Mutex::new(s);
        protobuf.services.push(m);
    }
    let b = check_for_dup_urls( & protobuf, "unknown".to_string());
    assert_eq ! (b, false, "not found");
    let c = check_for_dup_urls( & protobuf, "url-1".to_string());
    assert_eq ! (c, true, "was found");
}

// Enum to match protobuf enum for status
pub enum StatusEnum {
    SUCCESS   = 0,  // successful result
    NOTFOUND  = 1,  // matching protobuf not found
    DUPLICATE = 2,  // protobuf with duplicate url
    BADTOKEN  = 3,  // Invalid auth token
    AUTHERROR = 4,  // token create error
    SERVERROR = 5,  // server error
}

// Common routine to make a status packet
pub fn make_status_packet(code: StatusEnum,  error_message: String) -> registry::StatusPacket {
    let mapped_code = code as i32;
    let p = registry::StatusPacket {
        code: mapped_code,
        error_message: error_message,
    };
    p
}



