/*
*  kvsd store actions Module
*  SPDX-License-Identifier: MIT
*  Copyright (C) 2020 Benjamin Schilling
*/

// kvs modules
use crate::grpc::kvs_api::KeyValuePair;

// Available Actions
pub const ACTION_STORE: u8 = 0;
pub const ACTION_DELETE: u8 = 1;

// Action for the two_lock_queue
pub struct QueueAction {
    pub kv: KeyValuePair,
    pub action: u8,
}
