// Copyright (c) 2022, Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use sui_sdk::crypto::{AccountKeystore, FileBasedKeystore};
use sui_types::{
    base_types::SuiAddress,
    crypto::{AccountKeyPair, KeypairTraits, SuiKeyPair},
};

use std::path::PathBuf;

pub fn get_ed25519_keypair_from_keystore(
    keystore_path: PathBuf,
    requested_address: &SuiAddress,
) -> Result<AccountKeyPair> {
    let keystore = FileBasedKeystore::new(&keystore_path)?;
    match keystore.get_key(requested_address) {
        Ok(SuiKeyPair::Ed25519SuiKeyPair(kp)) => Ok(kp.copy()),
        _ => Err(anyhow::anyhow!("Unsupported key type")),
    }
}
