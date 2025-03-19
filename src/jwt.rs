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

use std::fs::read_to_string;
use jwt_simple::prelude::*;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MyAdditionalData {
    pub user_name: String,
    pub user_is_admin: bool,
    pub user_country: String,
}

// Create the jwt from the key pair
pub fn create_token(kp: &RS256KeyPair, username: String, subject: String, is_admin: bool,
    duration: Duration) -> Result<String, String> {

    let my_additional_data = MyAdditionalData {
        user_name: username,
        user_is_admin: is_admin,
        user_country: "US".to_string(),
    };

    let mut options = VerificationOptions::default();
    // Accept tokens that will only be valid in the future
    options.accept_future = true;
    // Accept tokens even if they have expired up to 15 minutes after the deadline,
    // and/or they will be valid within 15 minutes.
    // Note that 15 minutes is the default, since it is very common for clocks to be slightly off.
    options.time_tolerance = Some(Duration::from_mins(15));
    // Reject tokens if they were issued more than 1 hour ago
    options.max_validity = Some(Duration::from_mins(60));

    let claims = Claims::with_custom_claims(my_additional_data,
                                            duration).with_subject(subject);
    let token = match kp.sign(claims) {
        Ok(v) => v,
        Err(e) => {
            let msg = format!("failed to create token: {:?}", e);
            return Err(msg);
        },
    };
    Ok(token)
}

// Validate that the token is valid and correct
pub fn validate_token(kp: &RS256KeyPair, token: String) -> Result<JWTClaims<MyAdditionalData>, String> {

    if token.len() < 1 {
        let msg = "empty token, failed to validatation failed.".to_string();
        return Err(msg);
    }
    let mut options = VerificationOptions::default();
    // Accept tokens that will only be valid in the future
    options.accept_future = true;
    // Accept tokens even if they have expired up to 15 minutes after the deadline,
    // and/or they will be valid within 15 minutes.
    // Note that 15 minutes is the default, since it is very common for clocks to be slightly off.
    options.time_tolerance = Some(Duration::from_mins(15));
    // Reject tokens if they were issued more than 1 hour ago
    options.max_validity = Some(Duration::from_mins(60));

    let pk = kp.public_key();
    let x = match pk.verify_token::<MyAdditionalData>(token.as_str(), Some(options)) {
        Ok(v) => v,
        Err(e) => {
            let msg = format!("failed to validate token: {:?}", e);
            return Err(msg);
        },
    };
    return Ok(x)
}

// Load the PEM from disk and return the public key.
pub fn load_pem(file: String) -> Result<RS256KeyPair, String> {
    let pem = match read_to_string(&file) {
        Ok(v) => v,
        Err(e) => {
            let msg = format!("failed to read from file '{}': {:?}", file, e);
            return Err(msg);
        },
    };
    let kp = RS256KeyPair::from_pem(&pem).unwrap();
    Ok(kp)
}

#[test]
fn test_loadpem () {
    let kp = load_pem("mykey.pem".to_string()).unwrap();
    let kpp = kp.clone();
    let duration = Duration::from_mins(10);
    let token = create_token(&kp, "bill".to_string(),
                                  "mysubject".to_string(), false, duration).unwrap();
    let r = validate_token(&kpp, token);
    let rr = r.clone();
    if r.is_err() {
        assert_eq!("".to_string(), r.unwrap_err());
        println!("Error: {}", rr.unwrap_err());
    }
}

#[test]
fn test_validate_token() {
    // first create the keypair and make the token
    let kpr = RS256KeyPair::generate(4096);
    let kp = kpr.unwrap();
    let k2 = kp.clone();

    let r = create_token(&kp, "myuser".to_string(),
                         "mysubject".to_string(), false,
                         Duration::from_mins(1)
                        );
    if r.is_err() {

    }
    let token = r.unwrap().to_string();

    // Now validate
    let x = validate_token(&k2, token);
    if x.is_ok() {
        let z = x.unwrap();
        assert_eq!(z.custom.user_name, "myuser".to_string(), "mismatched user");
    }
    else {
        println!("error in validation {}", x.unwrap_err());
    }
}
