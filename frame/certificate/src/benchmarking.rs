// This file is part of Nuchain.
//
// Copyright (C) 2021 Rantai Nusantara Foundation.
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

//! Pallet Certificate pallet benchmarking

#![cfg(feature = "runtime-benchmarks")]

use super::*;

use frame_benchmarking::{account, benchmarks, whitelisted_caller};
use frame_support::assert_ok;
use frame_system::{EventRecord, RawOrigin};
use sp_core::crypto::UncheckedFrom;
use sp_runtime::traits::{Bounded, One};
use sp_std::vec;

use crate::Module as Certificate;
use pallet_organization::Module as Organization;

fn assert_last_event<T: Config>(generic_event: <T as Config>::Event) {
	let events = frame_system::Module::<T>::events();
	let system_event: <T as frame_system::Config>::Event = generic_event.into();
	// compare to the last event record
	let EventRecord { event, .. } = &events[events.len() - 1];
	assert_eq!(event, &system_event);
}

const ORG_NAME: &[u8] = b"org1";
const ORG_DESC: &[u8] = b"org1 desc";
const WEBSITE: &[u8] = b"https://some.org";
const EMAIL: &[u8] = b"info@some.org";

fn setup_org<T: Config>(caller: &T::AccountId) -> T::AccountId
where
	T::AccountId: UncheckedFrom<T::Hash> + AsRef<[u8]>,
{
	// assert_ok!(Organization::<T>::create(
	//     RawOrigin::Signed(caller.clone()).into(),
	//     ORG_NAME.to_vec(),
	//     ORG_DESC.to_vec(),
	//     caller.clone(),
	//     WEBSITE.to_vec(),
	//     EMAIL.to_vec(),
	//     None,
	// ));
	// let org_id = pallet_organization::OrganizationIndexOf::<T>::get(1).unwrap();
	// org_id

	// mock organization
	let org_id: T::AccountId = account("org1", 0, 0);
	pallet_organization::Organizations::<T>::insert(
		&org_id,
		pallet_organization::Organization {
			id: org_id.clone(),
			name: ORG_NAME.to_vec(),
			description: ORG_DESC.to_vec(),
			admin: caller.clone(),
			website: WEBSITE.to_vec(),
			email: EMAIL.to_vec(),
			suspended: false,
			block: T::BlockNumber::one(),
			timestamp: <T as pallet_organization::Config>::Time::now(),
			props: None,
		},
	);
	org_id
}

fn setup<T: Config>() -> (T::AccountId, T::AccountId)
where
	T::AccountId: UncheckedFrom<T::Hash> + AsRef<[u8]>,
{
	let caller: T::AccountId = whitelisted_caller();
	let org_id = setup_org::<T>(&caller);
	(caller, org_id)
}

const SIGNER: &[u8] = b"Grohl";

impl<T: Encode + Decode + Debug + Clone + Eq + PartialEq> CertDetail<T> {
	fn new(org_id: T) -> Self {
		CertDetail::<T> {
			name: b"CERT1".to_vec(),
			description: b"CERT1 desc".to_vec(),
			org_id,
			signer_name: None,
		}
	}

	fn signer(mut self, signer_name: Text) -> Self {
		self.signer_name = Some(signer_name);
		self
	}
}

benchmarks! {
	where_clause { where
		T: pallet_timestamp::Config,
		T::AccountId: UncheckedFrom<T::Hash> + AsRef<[u8]>,
	}

	create {
		let (caller, org_id) = setup::<T>();
		let cert_detail = CertDetail::new(org_id).signer(SIGNER.to_vec());
	}: _(RawOrigin::Signed(caller), cert_detail)

	issue {
		let (caller, org_id) = setup::<T>();
		let cert_detail:CertDetail<T::AccountId> = CertDetail::<T::AccountId>::new(org_id.clone()).signer(SIGNER.to_vec());
		let cert_id:CertId = Certificate::<T>::generate_hash(cert_detail.encode());
		Certificates::<T>::insert(cert_id, cert_detail);
		let now = <T as pallet::Config>::Time::now();
	}: _(RawOrigin::Signed(caller), org_id, cert_id, b"cert/01".to_vec(), b"Bob".to_vec(), None, None, Some(now))

	revoke {
		let (caller, org_id) = setup::<T>();

		let cert_detail:CertDetail<T::AccountId> = CertDetail::<T::AccountId>::new(org_id.clone()).signer(SIGNER.to_vec());
		let cert_id:CertId = Certificate::<T>::generate_hash(cert_detail.encode());
		Certificates::<T>::insert(cert_id, cert_detail);
		let now = <T as pallet::Config>::Time::now();
		let human_id = b"cert/01".to_vec();
		let recipient = b"Bob".to_vec();

		let _ = Certificate::<T>::issue(RawOrigin::Signed(caller.clone()).into(), org_id.clone(), cert_id,
			human_id.clone(),
			recipient.clone(), None, None, Some(now));
		let issued_id = [1u8; 11];

		let proof = CertProof {
			cert_id,
			human_id,
			recipient,
			time: now,
			expired: None,
			revoked: false,
			block: T::BlockNumber::one(),
			signer_name: None,
			props: None,
		};
		IssuedCert::<T>::insert(&issued_id, proof);
		IssuedCertOwner::<T>::insert(&org_id, &caller, vec![issued_id.clone()]);

	}: _(RawOrigin::Signed(caller), org_id, issued_id, true)
}
