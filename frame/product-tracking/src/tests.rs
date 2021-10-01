// This file is part of Nuchain.
//
// Copyright (C) 2021 Rantai Nusantara Foundation.
// SPDX-License-Identifier: Apache-2.0
//
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

use super::*;
use crate::{
    mock::{
        account_key, new_test_ext, Event as TestEvent, Origin, ProductRegistry, ProductTracking,
        System, Test, Timestamp,
    },
    types::*,
    Error,
};
use fixed::types::I16F16;
use frame_support::{assert_err_ignore_postinfo, assert_noop, assert_ok, dispatch};

pub fn store_test_tracking<T: Config>(
    id: TrackingId,
    owner: T::AccountId,
    status: TrackingStatus,
    products: Vec<ProductId>,
    registered: T::Moment,
) {
    Tracking::<T>::insert(
        id.clone(),
        Track {
            id,
            owner,
            status,
            products,
            registered,
            updated: None,
            parent_id: None,
            props: None,
        },
    );
}

pub fn store_test_event<T: Config>(
    tracking_id: TrackingId,
    event_type: TrackingEventType,
    status: TrackingStatus,
) {
    let event = TrackingEvent {
        event_type,
        tracking_id: tracking_id.clone(),
        location: None,
        readings: vec![],
        status: status.clone(),
        timestamp: pallet_timestamp::Module::<T>::now(),
        props: None,
    };
    let event_idx = <EventCount<T>>::get()
        .unwrap_or(0)
        .checked_add(1)
        .expect("not overflow");
    <EventCount<T>>::put(event_idx);
    AllEvents::<T>::insert(event_idx, event);
    <EventsOfTracking<T>>::append(tracking_id, event_idx);
}

const TEST_PRODUCT_ID: &str = "00012345678905";
const TEST_TRACKING_ID: &str = "0001";
const TEST_ORGANIZATION: &str = "Northwind";
const TEST_SENDER: &str = "Alice";
const LONG_VALUE : &str = "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Donec aliquam ut tortor nec congue. Pellente";

const STATUS_EMPTY: &[u8] = b"";
const STATUS_PENDING: &[u8] = b"Pending";
const STATUS_QA_CHECK: &[u8] = b"QA check";
// const STATUS_PROCESSING: &[u8] = b"Processing";
const STATUS_DELIVER: &[u8] = b"Deliver";
const STATUS_IN_TRANSIT: &[u8] = b"In Transit";

const YEAR1: u32 = 2020;
const YEAR2: u32 = 2021;

fn with_account<F>(func: F)
where
    F: FnOnce(
        <Test as frame_system::Config>::AccountId,
        <Test as frame_system::Config>::AccountId,
        <Test as pallet_timestamp::Config>::Moment,
    ),
{
    new_test_ext().execute_with(|| {
        let sender = account_key(TEST_SENDER);
        let org = account_key(TEST_ORGANIZATION);
        let now = 42;
        Timestamp::set_timestamp(now);

        func(sender, org, now);
    });
}

fn with_account_and_org<F>(func: F)
where
    F: FnOnce(
        <Test as frame_system::Config>::AccountId,
        <Test as frame_system::Config>::AccountId,
        <Test as pallet_timestamp::Config>::Moment,
    ),
{
    new_test_ext().execute_with(|| {
        let sender = account_key(TEST_SENDER);
        let org = account_key(TEST_ORGANIZATION);

        // mock organization
        pallet_organization::Organizations::<Test>::insert(
            org.clone(),
            pallet_organization::Organization {
                id: org.clone(),
                name: TEST_ORGANIZATION.as_bytes().to_vec(),
                description: vec![],
                admin: sender.clone(),
                website: vec![],
                email: vec![],
                suspended: false,
                props: None,
            },
        );
        // Make sender as org owner
        <pallet_did::Module<Test>>::set_owner(&sender, &org, &sender);

        let now = 42;
        Timestamp::set_timestamp(now);

        func(sender, org, now);
    });
}

#[test]
fn non_org_owner_cannot_register() {
    with_account(|sender, org, _now| {
        let id = TEST_TRACKING_ID.as_bytes().to_owned();
        assert_noop!(
            ProductTracking::register(
                Origin::signed(sender),
                id.clone(),
                org.clone(),
                YEAR1,
                vec![],
                None,
                None
            ),
            pallet_organization::Error::<Test>::NotExists
        );
    });
}

#[test]
fn test_register_with_props() {
    with_account_and_org(|sender, org, now| {
        let id = TEST_TRACKING_ID.as_bytes().to_owned();

        let props = Some(vec![Property::new(b"key", b"something")]);

        let result = ProductTracking::register(
            Origin::signed(sender),
            id.clone(),
            org.clone(),
            YEAR1,
            vec![],
            None,
            props.clone(),
        );

        assert_ok!(result);

        assert_eq!(
            ProductTracking::tracking(&id),
            Some(Track {
                id: id.clone(),
                owner: org,
                status: STATUS_EMPTY.to_vec(),
                products: vec![],
                registered: now,
                updated: None,
                parent_id: None,
                props
            })
        );
    })
}

#[test]
fn test_register_with_invalid_props() {
    with_account_and_org(|sender, org, _now| {
        let id = TEST_TRACKING_ID.as_bytes().to_owned();

        assert_noop!(
            ProductTracking::register(
                Origin::signed(sender),
                id.clone(),
                org.clone(),
                YEAR1,
                vec![],
                None,
                Some(vec![Property::new(b"0123456789012345678901234567891", b"12345")]),
            ),
            Error::<Test>::InvalidPropName
        );

        assert_noop!(
            ProductTracking::register(
                Origin::signed(sender),
                id.clone(),
                org.clone(),
                YEAR1,
                vec![],
                None,
                Some(vec![Property::new(b"", b"12345")]),
            ),
            Error::<Test>::InvalidPropName
        );

        assert_noop!(
            ProductTracking::register(
                Origin::signed(sender),
                id.clone(),
                org.clone(),
                YEAR1,
                vec![],
                None,
                Some(vec![Property::new(
                    b"12345",
                    b"0123456789012345678901234567890123456789012345678901234567891"
                )]),
            ),
            Error::<Test>::InvalidPropValue
        );

        assert_noop!(
            ProductTracking::register(
                Origin::signed(sender),
                id.clone(),
                org.clone(),
                YEAR1,
                vec![],
                None,
                Some(vec![Property::new(b"12345", b"")]),
            ),
            Error::<Test>::InvalidPropValue
        );

        assert_noop!(
            ProductTracking::register(
                Origin::signed(sender),
                id.clone(),
                org.clone(),
                YEAR1,
                vec![],
                None,
                Some(vec![
                    // 6x
                    Property::new(b"12345", b"123456789012345678901"),
                    Property::new(b"12345", b"123456789012345678901"),
                    Property::new(b"12345", b"123456789012345678901"),
                    Property::new(b"12345", b"123456789012345678901"),
                    Property::new(b"12345", b"123456789012345678901"),
                    Property::new(b"12345", b"123456789012345678901")
                ]),
            ),
            Error::<Test>::TooManyProps
        );
    })
}

#[test]
fn register_without_products() {
    with_account_and_org(|sender, org, now| {
        let id = TEST_TRACKING_ID.as_bytes().to_owned();

        let result = ProductTracking::register(
            Origin::signed(sender),
            id.clone(),
            org.clone(),
            YEAR1,
            vec![],
            None,
            None,
        );

        assert_ok!(result);

        assert_eq!(
            ProductTracking::tracking(&id),
            Some(Track {
                id: id.clone(),
                owner: org,
                status: STATUS_EMPTY.to_vec(),
                products: vec![],
                registered: now,
                updated: None,
                parent_id: None,
                props: None
            })
        );

        assert_eq!(
            <TrackingOfOrganization<Test>>::get(org, YEAR1),
            Some(vec![id.clone()])
        );

        assert!(System::events().iter().any(|er| er.event
            == TestEvent::pallet_product_tracking(Event::TrackingRegistered(
                sender,
                id.clone(),
                org
            ))));
    });
}

#[test]
fn cannot_register_non_existing_product() {
    with_account_and_org(|sender, org, _now| {
        let id = TEST_TRACKING_ID.as_bytes().to_owned();

        let result = ProductTracking::register(
            Origin::signed(sender),
            id.clone(),
            org.clone(),
            YEAR2,
            vec![
                b"00012345600001".to_vec(),
                b"00012345600002".to_vec(),
                b"00012345600003".to_vec(),
            ],
            None,
            None,
        );

        assert_err_ignore_postinfo!(result, Error::<Test>::ProductNotExists);
    });
}

/// This function only mocking product, bypass all validation
fn register_products(prod_ids: &Vec<Vec<u8>>, org_id: &<Test as frame_system::Config>::AccountId) {
    for prod_id in prod_ids {
        let product = ProductRegistry::new_product()
            .identified_by(prod_id.to_vec())
            .owned_by(org_id.clone())
            .registered_on(Timestamp::now())
            .with_props(Some(vec![]))
            .build();
        pallet_product_registry::Products::<Test>::insert(prod_id.to_vec(), product);
    }
}

#[test]
fn register_with_valid_products() {
    with_account_and_org(|sender, org, now| {
        let id = TEST_TRACKING_ID.as_bytes().to_owned();

        let products = vec![
            b"00012345600001".to_vec(),
            b"00012345600002".to_vec(),
            b"00012345600003".to_vec(),
        ];

        register_products(&products, &org);

        let result = ProductTracking::register(
            Origin::signed(sender),
            id.clone(),
            org.clone(),
            YEAR2,
            products,
            None,
            None,
        );

        assert_ok!(result);

        assert_eq!(
            ProductTracking::tracking(&id),
            Some(Track {
                id: id.clone(),
                owner: org,
                status: STATUS_EMPTY.to_vec(),
                products: vec![
                    b"00012345600001".to_vec(),
                    b"00012345600002".to_vec(),
                    b"00012345600003".to_vec(),
                ],
                registered: now,
                updated: None,
                parent_id: None,
                props: None
            })
        );

        assert_eq!(
            <TrackingOfOrganization<Test>>::get(org, YEAR2),
            Some(vec![id.clone()])
        );

        assert!(System::events().iter().any(|er| er.event
            == TestEvent::pallet_product_tracking(Event::TrackingRegistered(
                sender,
                id.clone(),
                org
            ))));
    });
}

#[test]
fn register_with_invalid_sender() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            ProductTracking::register(
                Origin::none(),
                TEST_TRACKING_ID.as_bytes().to_owned(),
                account_key(TEST_ORGANIZATION),
                YEAR1,
                vec!(),
                None,
                None
            ),
            dispatch::DispatchError::BadOrigin
        );
    });
}

#[test]
fn register_with_missing_id() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            ProductTracking::register(
                Origin::signed(account_key(TEST_SENDER)),
                vec!(),
                account_key(TEST_ORGANIZATION),
                YEAR1,
                vec!(),
                None,
                None
            ),
            Error::<Test>::InvalidOrMissingIdentifier
        );
    });
}

#[test]
fn register_with_long_id() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            ProductTracking::register(
                Origin::signed(account_key(TEST_SENDER)),
                LONG_VALUE.as_bytes().to_owned(),
                account_key(TEST_ORGANIZATION),
                YEAR1,
                vec!(),
                None,
                None
            ),
            Error::<Test>::InvalidOrMissingIdentifier
        );
    })
}

#[test]
fn register_with_existing_id() {
    with_account_and_org(|_sender, org, _now| {
        let existing_tracking = TEST_TRACKING_ID.as_bytes().to_owned();

        assert_ok!(ProductTracking::register(
            Origin::signed(account_key(TEST_SENDER)),
            existing_tracking.clone(),
            org,
            YEAR1,
            vec![],
            None,
            None
        ));

        assert_noop!(
            ProductTracking::register(
                Origin::signed(account_key(TEST_SENDER)),
                existing_tracking,
                org,
                YEAR1,
                vec![],
                None,
                None
            ),
            Error::<Test>::TrackingAlreadyExists
        );
    })
}

#[test]
fn register_with_too_many_products() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            ProductTracking::register(
                Origin::signed(account_key(TEST_SENDER)),
                TEST_TRACKING_ID.as_bytes().to_owned(),
                account_key(TEST_ORGANIZATION),
                YEAR1,
                vec![
                    b"00012345600001".to_vec(),
                    b"00012345600002".to_vec(),
                    b"00012345600003".to_vec(),
                    b"00012345600004".to_vec(),
                    b"00012345600005".to_vec(),
                    b"00012345600006".to_vec(),
                    b"00012345600007".to_vec(),
                    b"00012345600008".to_vec(),
                    b"00012345600009".to_vec(),
                    b"00012345600010".to_vec(),
                    b"00012345600011".to_vec(),
                ],
                None,
                None
            ),
            Error::<Test>::TrackingHasTooManyProducts
        );
    })
}

#[test]
fn update_status_with_invalid_sender() {
    new_test_ext().execute_with(|| {
        let now = 42;

        assert_noop!(
            ProductTracking::update_status(
                Origin::none(),
                TEST_TRACKING_ID.as_bytes().to_owned(),
                STATUS_QA_CHECK.to_vec(),
                now,
                None,
                None,
                None
            ),
            dispatch::DispatchError::BadOrigin
        );
    });
}

#[test]
fn update_status_with_custom_props_works() {
    // test ini memastikan bahwa update status yang menyertakan props
    // akan menambahkan properties di TrackingEvent (bukan di Tracking object-nya)
    with_account_and_org(|sender, org, now| {
        let tracking_id = TEST_TRACKING_ID.as_bytes().to_owned();

        store_test_tracking::<Test>(
            tracking_id.clone(),
            org,
            STATUS_PENDING.to_vec(),
            vec![TEST_PRODUCT_ID.as_bytes().to_owned()],
            now,
        );

        store_test_event::<Test>(
            tracking_id.clone(),
            TrackingEventType::TrackingRegistration,
            b"registered".to_vec(),
        );

        assert_ok!(ProductTracking::update_status(
            Origin::signed(sender),
            TEST_TRACKING_ID.as_bytes().to_owned(),
            STATUS_QA_CHECK.to_vec(),
            now,
            None,
            None,
            Some(vec![Property::new(b"satu", b"001")])
        ));

        let event_index = ProductTracking::events_of_tracking(&tracking_id)
            .and_then(|mut a| a.pop())
            .expect("event index");

        assert_eq!(
            ProductTracking::event_by_idx(event_index).and_then(|ev| ev.props),
            Some(vec![Property::new(b"satu", b"001")])
        );
    });
}

#[test]
fn update_status_with_custom_props_invalid() {
    // test ini memastikan bahwa update status yang menyertakan props
    // tetapi invalid props-nya
    with_account_and_org(|sender, org, now| {
        let tracking_id = TEST_TRACKING_ID.as_bytes().to_owned();

        store_test_tracking::<Test>(
            tracking_id.clone(),
            org,
            STATUS_PENDING.to_vec(),
            vec![TEST_PRODUCT_ID.as_bytes().to_owned()],
            now,
        );

        store_test_event::<Test>(
            tracking_id.clone(),
            TrackingEventType::TrackingRegistration,
            b"registered".to_vec(),
        );

        assert_noop!(
            ProductTracking::update_status(
                Origin::signed(sender),
                TEST_TRACKING_ID.as_bytes().to_owned(),
                STATUS_QA_CHECK.to_vec(),
                now,
                None,
                None,
                Some(vec![Property::new(b"", b"001")])
            ),
            Error::<Test>::InvalidPropName
        );
    });
}

#[test]
fn update_status_with_missing_tracking_id() {
    new_test_ext().execute_with(|| {
        let now = 42;

        assert_noop!(
            ProductTracking::update_status(
                Origin::signed(account_key(TEST_SENDER)),
                vec![],
                STATUS_QA_CHECK.to_vec(),
                now,
                None,
                None,
                None
            ),
            Error::<Test>::InvalidOrMissingIdentifier
        );
    });
}

#[test]
fn update_status_with_long_tracking_id() {
    new_test_ext().execute_with(|| {
        let now = 42;

        assert_noop!(
            ProductTracking::update_status(
                Origin::signed(account_key(TEST_SENDER)),
                LONG_VALUE.as_bytes().to_owned(),
                STATUS_QA_CHECK.to_vec(),
                now,
                None,
                None,
                None
            ),
            Error::<Test>::InvalidOrMissingIdentifier,
        );
    });
}

#[test]
fn update_status_with_unknown_tracking() {
    new_test_ext().execute_with(|| {
        let unknown_tracking = TEST_TRACKING_ID.as_bytes().to_owned();
        let now = 42;

        assert_noop!(
            ProductTracking::update_status(
                Origin::signed(account_key(TEST_SENDER)),
                unknown_tracking,
                STATUS_QA_CHECK.to_vec(),
                now,
                None,
                None,
                None
            ),
            Error::<Test>::TrackingIsUnknown,
        );
    })
}

#[test]
fn update_status_pickup() {
    new_test_ext().execute_with(|| {
        let owner = account_key(TEST_ORGANIZATION);
        let tracking_id = TEST_TRACKING_ID.as_bytes().to_owned();
        let now = 64;
        Timestamp::set_timestamp(now);

        // Store tracking w/ Pending status
        store_test_tracking::<Test>(
            tracking_id.clone(),
            owner,
            STATUS_PENDING.to_vec(),
            vec![TEST_PRODUCT_ID.as_bytes().to_owned()],
            now,
        );

        // Store shipping registration event
        store_test_event::<Test>(
            tracking_id.clone(),
            TrackingEventType::TrackingRegistration,
            b"registered".to_vec(),
        );

        // Dispatchable call succeeds
        assert_ok!(ProductTracking::update_status(
            Origin::signed(owner),
            tracking_id.clone(),
            STATUS_QA_CHECK.to_vec(),
            now,
            None,
            None,
            None
        ));

        // Storage is correctly updated
        assert_eq!(ProductTracking::event_count(), Some(2));
        assert_eq!(
            AllEvents::<Test>::get(2),
            Some(TrackingEvent {
                event_type: TrackingEventType::TrackingUpdateStatus,
                tracking_id: tracking_id.clone(),
                location: None,
                readings: vec![],
                status: STATUS_QA_CHECK.to_vec(),
                timestamp: now,
                props: None
            })
        );
        assert_eq!(
            ProductTracking::events_of_tracking(&tracking_id),
            Some(vec![1, 2])
        );

        // Tracking's status should be updated to 'InTransit'
        assert_eq!(
            ProductTracking::tracking(&tracking_id),
            Some(Track {
                id: tracking_id.clone(),
                owner: owner,
                status: STATUS_QA_CHECK.to_vec(),
                products: vec![TEST_PRODUCT_ID.as_bytes().to_owned()],
                registered: now,
                updated: Some(now),
                parent_id: None,
                props: None
            })
        );

        // Event is raised
        assert!(System::events().iter().any(|er| er.event
            == TestEvent::pallet_product_tracking(Event::TrackingStatusUpdated(
                owner,
                tracking_id.clone(),
                2,
                STATUS_QA_CHECK.to_vec(),
            ))));
    })
}

#[test]
fn update_status_delivery() {
    new_test_ext().execute_with(|| {
        let owner = account_key(TEST_ORGANIZATION);
        let tracking_id = TEST_TRACKING_ID.as_bytes().to_owned();
        let now = Timestamp::now();

        // Store tracking w/ InTransit status
        store_test_tracking::<Test>(
            tracking_id.clone(),
            owner,
            STATUS_PENDING.to_vec(),
            vec![TEST_PRODUCT_ID.as_bytes().to_owned()],
            now,
        );

        // Store shipping registration & pickup events
        store_test_event::<Test>(
            tracking_id.clone(),
            TrackingEventType::TrackingRegistration,
            b"".to_vec(),
        );
        // store_test_event::<Test>(tracking_id.clone(), TrackingEventType::TrackingPickup);

        // Dispatchable call succeeds
        assert_ok!(ProductTracking::update_status(
            Origin::signed(owner),
            tracking_id.clone(),
            STATUS_DELIVER.to_vec(),
            now,
            None,
            None,
            None
        ));

        // Storage is correctly updated
        assert_eq!(ProductTracking::event_count(), Some(2));
        assert_eq!(
            AllEvents::<Test>::get(2),
            Some(TrackingEvent {
                event_type: TrackingEventType::TrackingUpdateStatus,
                tracking_id: tracking_id.clone(),
                location: None,
                readings: vec![],
                status: STATUS_DELIVER.to_vec(),
                timestamp: now,
                props: None
            })
        );
        assert_eq!(
            ProductTracking::events_of_tracking(&tracking_id),
            Some(vec![1, 2])
        );

        // Tracking's status should be updated to 'Delivered'
        // and updated timestamp updated
        assert_eq!(
            ProductTracking::tracking(&tracking_id),
            Some(Track {
                id: tracking_id.clone(),
                owner: owner,
                status: STATUS_DELIVER.to_vec(),
                products: vec![TEST_PRODUCT_ID.as_bytes().to_owned()],
                registered: now,
                updated: Some(now),
                parent_id: None,
                props: None
            })
        );

        // Events is raised
        assert!(System::events().iter().any(|er| er.event
            == TestEvent::pallet_product_tracking(Event::TrackingStatusUpdated(
                owner,
                tracking_id.clone(),
                2,
                STATUS_DELIVER.to_vec()
            ))));
    })
}

#[test]
fn monitor_tracking_with_negative_latlon() {
    new_test_ext().execute_with(|| {
        let owner = account_key(TEST_ORGANIZATION);
        let tracking_id = TEST_TRACKING_ID.as_bytes().to_owned();
        let now = 55;
        Timestamp::set_timestamp(now);

        // Store tracking w/ InTransit status
        store_test_tracking::<Test>(
            tracking_id.clone(),
            owner,
            STATUS_IN_TRANSIT.to_vec(),
            vec![TEST_PRODUCT_ID.as_bytes().to_owned()],
            now,
        );

        // Store shipping registration & pickup events
        store_test_event::<Test>(
            tracking_id.clone(),
            TrackingEventType::TrackingRegistration,
            b"".to_vec(),
        );
        // store_test_event::<Test>(tracking_id.clone(), TrackingEventType::TrackingPickup);

        // Define location & readings for sensor reading
        let location = ReadPoint {
            // Rio de Janeiro, Brazil
            latitude: b"-22.9466369".to_vec(),
            longitude: b"-43.233472".to_vec(),
        };

        let readings = vec![Reading {
            device_id: "14d453ea4bdf46bc8042".as_bytes().to_owned(),
            reading_type: ReadingType::Temperature,
            value: b"20.123".to_vec(),
            timestamp: now,
        }];

        // Dispatchable call succeeds
        assert_ok!(ProductTracking::update_status(
            Origin::signed(owner),
            tracking_id.clone(),
            STATUS_QA_CHECK.to_vec(),
            now,
            Some(location.clone()),
            Some(readings.clone()),
            None
        ));

        // Storage is correctly updated
        assert_eq!(ProductTracking::event_count(), Some(2));
        assert_eq!(
            AllEvents::<Test>::get(2),
            Some(TrackingEvent {
                event_type: TrackingEventType::TrackingUpdateStatus,
                tracking_id: tracking_id.clone(),
                location: Some(location),
                readings: readings,
                status: STATUS_QA_CHECK.to_vec(),
                timestamp: now,
                props: None
            })
        );
        assert_eq!(
            ProductTracking::events_of_tracking(&tracking_id),
            Some(vec![1, 2])
        );

        // Tracking's status should still be 'InTransit'
        assert_eq!(
            ProductTracking::tracking(&tracking_id),
            Some(Track {
                id: tracking_id.clone(),
                owner: owner,
                status: STATUS_QA_CHECK.to_vec(),
                products: vec![TEST_PRODUCT_ID.as_bytes().to_owned()],
                registered: now,
                updated: Some(now),
                parent_id: None,
                props: None
            })
        );
    })
}

#[test]
fn non_org_owner_cannot_update_status() {
    with_account(|sender, org, now| {
        let id = TEST_TRACKING_ID.as_bytes().to_owned();

        // Store tracking w/ Pending status
        store_test_tracking::<Test>(
            id.clone(),
            org,
            STATUS_PENDING.to_vec(),
            vec![TEST_PRODUCT_ID.as_bytes().to_owned()],
            now,
        );

        // Store shipping registration event
        store_test_event::<Test>(
            id.clone(),
            TrackingEventType::TrackingRegistration,
            b"".to_vec(),
        );

        assert_err_ignore_postinfo!(
            ProductTracking::update_status(
                Origin::signed(sender),
                id.clone(),
                STATUS_QA_CHECK.to_vec(),
                now,
                None,
                None,
                None
            ),
            Error::<Test>::PermissionDenied
        );
    });
}

#[test]
fn hacker_cannot_update_status() {
    with_account_and_org(|_sender, org, now| {
        let id = TEST_TRACKING_ID.as_bytes().to_owned();

        // Store tracking w/ Pending status
        store_test_tracking::<Test>(
            id.clone(),
            org,
            STATUS_PENDING.to_vec(),
            vec![TEST_PRODUCT_ID.as_bytes().to_owned()],
            now,
        );

        // Store shipping registration event
        store_test_event::<Test>(
            id.clone(),
            TrackingEventType::TrackingRegistration,
            b"".to_vec(),
        );

        assert_noop!(
            ProductTracking::update_status(
                Origin::signed(account_key("Hacker")),
                id.clone(),
                STATUS_DELIVER.to_vec(),
                now,
                None,
                None,
                None
            ),
            Error::<Test>::PermissionDenied
        );
    });
}

#[test]
fn delegated_account_can_update_status() {
    with_account_and_org(|sender, org, now| {
        let id = TEST_TRACKING_ID.as_bytes().to_owned();

        // Store tracking w/ Pending status
        store_test_tracking::<Test>(
            id.clone(),
            org,
            STATUS_PENDING.to_vec(),
            vec![TEST_PRODUCT_ID.as_bytes().to_owned()],
            now,
        );

        // Store shipping registration event
        store_test_event::<Test>(
            id.clone(),
            TrackingEventType::TrackingRegistration,
            b"".to_vec(),
        );

        let delegated = account_key("Wahid");

        // berikan akses ProductTracker kepada Wahid
        assert_ok!(pallet_organization::Module::<Test>::h_delegate_access_as(
            &sender,
            &org,
            &delegated,
            b"ProductTracker",
            None
        ));

        assert_ok!(ProductTracking::update_status(
            Origin::signed(delegated),
            id.clone(),
            STATUS_DELIVER.to_vec(),
            now,
            None,
            None,
            None
        ));
    });
}

#[test]
fn register_tracking_with_parent_id() {
    with_account_and_org(|sender, org, now| {
        let id = TEST_TRACKING_ID.as_bytes().to_owned();

        let props = Some(vec![Property::new(b"key", b"something")]);

        let parent_id = Some(b"tracking-prev-01".to_vec());

        let result = ProductTracking::register(
            Origin::signed(sender),
            id.clone(),
            org.clone(),
            YEAR1,
            vec![],
            parent_id.clone(),
            props.clone(),
        );

        assert_ok!(result);

        assert_eq!(
            ProductTracking::tracking(&id),
            Some(Track {
                id: id.clone(),
                owner: org,
                status: STATUS_EMPTY.to_vec(),
                products: vec![],
                registered: now,
                updated: None,
                parent_id,
                props
            })
        );
    })
}
