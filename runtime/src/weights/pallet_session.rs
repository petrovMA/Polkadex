// This file is part of Substrate.

// Copyright (C) 2017-2020 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 2.0.0-rc6

#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::weights::{constants::RocksDbWeight as DbWeight, Weight};

pub struct WeightInfo;

impl pallet_session::WeightInfo for WeightInfo {
	fn set_keys() -> Weight {
		88_411_000_u64
			.saturating_add(DbWeight::get().reads(6_u64))
			.saturating_add(DbWeight::get().writes(5_u64))
	}
	fn purge_keys() -> Weight {
		51_843_000_u64
			.saturating_add(DbWeight::get().reads(2_u64))
			.saturating_add(DbWeight::get().writes(5_u64))
	}
}
