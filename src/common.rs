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

///////////////////////////////////////////////////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////////////////
//// Caveats - Mutex objects are used to enforce concurrent safety. The
//// rational for these helper routines is that the mutex is in a locked
//// state before being called. This ensures higher level functions will
//// release locks based upon scoping.
///////////////////////////////////////////////////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////////////////

use std::sync::Mutex;
use crate::{Protobuf, Protobufs, Service};

pub fn make_service(url: String) -> Mutex<Service> {
    let s = Service{
        url: url,
        ctr: 0,
    };
    let m = Mutex::new(s);
    m
}

// Make a new protobuf group definition
pub fn make_protobuf(protobuf_name: &String) -> Mutex<Protobuf> {
    let p = Protobuf {
        name: protobuf_name.to_string(),
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
    if ( crate::find_protobuf(protobufs, protobuf_name)) .is_some() {
        return Err("Protobuf is already registered".to_string())
    }
    let p = crate::make_protobuf(&name);
    protobufs.protomap.insert(name2, p);
    Ok(())
}

// Add a new service definition to a protobuf grouping.
pub fn add_service(protobuf: &mut Protobuf, url: String) -> Result<(), String> {
    //if false { //check_for_dup_urls(&protobuf.services, url) == true {
    //Err(_s);
    //} else
    {
        let s = crate::make_service(url);
        //let aa = protobuf.unwrap();
        //let bb = aa.lock().unwrap();
        let mut cc = protobuf.services.lock().unwrap();
        cc.push(s);
    }
    Ok(())
}

// Check for duplicate protobuf urls in the protobuf group.
fn check_for_dup_urls(ids: &Vec<Mutex<Service>>, url: String) -> bool {
    if ids.len() > 0 {
        for x in 0..ids.len() {
            let s = ids[x].lock().unwrap();
            if url == s.url {
                return true;
            }
        }
    }
    return false;
}

