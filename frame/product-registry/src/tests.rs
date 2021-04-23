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

use crate::{
    self as pallet_product_registry, mock::*, Config, Error, Product, ProductId, ProductProperty,
    Products, ProductsOfOrganization,
};
use frame_support::{assert_err_ignore_postinfo, assert_noop, assert_ok, dispatch};

type PalletEvent = pallet_product_registry::Event<Test>;

pub fn store_test_product<T: Config>(id: ProductId, owner: T::AccountId, registered: T::Moment) {
    Products::<T>::insert(
        id.clone(),
        Product {
            id,
            owner,
            registered,
            props: None,
        },
    );
}

const TEST_PRODUCT_ID: &str = "00012345600012";
const TEST_ORGANIZATION: &str = "Northwind";
const TEST_SENDER: &str = "Alice";
const LONG_VALUE : &str = "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Donec aliquam ut tortor nec congue. Pellente";
const YEAR1: u32 = 2020;
const YEAR2: u32 = 2021;

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

        // Mock organization
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
            },
        );

        let now = 42;
        Timestamp::set_timestamp(now);

        func(sender, org, now);
    });
}

#[test]
fn create_product_without_props() {
    with_account_and_org(|sender, org, now| {
        let id = TEST_PRODUCT_ID.as_bytes().to_owned();

        let result =
            ProductRegistry::register(Origin::signed(sender), id.clone(), org.clone(), YEAR1, None);

        assert_ok!(result);

        assert_eq!(
            ProductRegistry::product_by_id(&id),
            Some(Product {
                id: id.clone(),
                owner: org,
                registered: now,
                props: None
            })
        );

        assert_eq!(
            <ProductsOfOrganization<Test>>::get(org, YEAR1),
            Some(vec![id.clone()])
        );

        assert_eq!(ProductRegistry::owner_of(&id), Some(org));

        // Event is raised
        assert!(System::events().iter().any(|er| er.event
            == Event::pallet_product_registry(PalletEvent::ProductRegistered(
                sender,
                id.clone(),
                org
            ))));
    });
}

#[test]
fn create_product_with_valid_props() {
    with_account_and_org(|sender, org, now| {
        let id = TEST_PRODUCT_ID.as_bytes().to_owned();

        let result = ProductRegistry::register(
            Origin::signed(sender),
            id.clone(),
            org.clone(),
            YEAR2,
            Some(vec![
                ProductProperty::new(b"prop1", b"val1"),
                ProductProperty::new(b"prop2", b"val2"),
                ProductProperty::new(b"prop3", b"val3"),
            ]),
        );

        assert_ok!(result);

        assert_eq!(
            ProductRegistry::product_by_id(&id),
            Some(Product {
                id: id.clone(),
                owner: org,
                registered: now,
                props: Some(vec![
                    ProductProperty::new(b"prop1", b"val1"),
                    ProductProperty::new(b"prop2", b"val2"),
                    ProductProperty::new(b"prop3", b"val3"),
                ]),
            })
        );

        assert_eq!(
            <ProductsOfOrganization<Test>>::get(&org, YEAR2),
            Some(vec![id.clone()])
        );

        assert_eq!(ProductRegistry::owner_of(&id), Some(org));

        // Event is raised
        assert!(System::events().iter().any(|er| er.event
            == Event::pallet_product_registry(PalletEvent::ProductRegistered(
                sender,
                id.clone(),
                org
            ))));
    });
}

#[test]
fn non_organization_account_cannot_register_product() {
    new_test_ext().execute_with(|| {
        let id = TEST_PRODUCT_ID.as_bytes().to_owned();
        let sender = account_key(TEST_SENDER);
        assert_err_ignore_postinfo!(
            ProductRegistry::register(
                Origin::signed(sender),
                id,
                account_key(TEST_ORGANIZATION),
                YEAR1,
                None
            ),
            pallet_organization::Error::<Test>::NotExists
        );
    });
}

#[test]
fn create_product_with_invalid_sender() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            ProductRegistry::register(
                Origin::none(),
                vec!(),
                account_key(TEST_ORGANIZATION),
                YEAR1,
                None
            ),
            dispatch::DispatchError::BadOrigin
        );
    });
}

#[test]
fn create_product_with_missing_id() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            ProductRegistry::register(
                Origin::signed(account_key(TEST_SENDER)),
                vec!(),
                account_key(TEST_ORGANIZATION),
                YEAR1,
                None
            ),
            Error::<Test>::ProductIdMissing
        );
    });
}

#[test]
fn create_product_with_long_id() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            ProductRegistry::register(
                Origin::signed(account_key(TEST_SENDER)),
                LONG_VALUE.as_bytes().to_owned(),
                account_key(TEST_ORGANIZATION),
                YEAR1,
                None
            ),
            Error::<Test>::ProductIdTooLong
        );
    })
}

#[test]
fn create_product_with_existing_id() {
    new_test_ext().execute_with(|| {
        let existing_product = TEST_PRODUCT_ID.as_bytes().to_owned();
        let now = 42;

        store_test_product::<Test>(
            existing_product.clone(),
            account_key(TEST_ORGANIZATION),
            now,
        );

        assert_noop!(
            ProductRegistry::register(
                Origin::signed(account_key(TEST_SENDER)),
                existing_product,
                account_key(TEST_ORGANIZATION),
                YEAR1,
                None
            ),
            Error::<Test>::ProductIdExists
        );
    })
}

#[test]
fn create_product_with_too_many_props() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            ProductRegistry::register(
                Origin::signed(account_key(TEST_SENDER)),
                TEST_PRODUCT_ID.as_bytes().to_owned(),
                account_key(TEST_ORGANIZATION),
                YEAR1,
                Some(vec![
                    ProductProperty::new(b"prop1", b"val1"),
                    ProductProperty::new(b"prop2", b"val2"),
                    ProductProperty::new(b"prop3", b"val3"),
                    ProductProperty::new(b"prop4", b"val4"),
                    ProductProperty::new(b"prop5", b"val5"),
                    ProductProperty::new(b"prop6", b"val6")
                ])
            ),
            Error::<Test>::TooManyProps
        );
    })
}

#[test]
fn create_product_with_invalid_prop_name() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            ProductRegistry::register(
                Origin::signed(account_key(TEST_SENDER)),
                TEST_PRODUCT_ID.as_bytes().to_owned(),
                account_key(TEST_ORGANIZATION),
                YEAR1,
                Some(vec![
                    ProductProperty::new(b"prop1", b"val1"),
                    ProductProperty::new(b"prop2", b"val2"),
                    ProductProperty::new(&LONG_VALUE.as_bytes().to_owned(), b"val3"),
                ])
            ),
            Error::<Test>::InvalidPropName
        );
    })
}

#[test]
fn create_product_with_invalid_prop_value() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            ProductRegistry::register(
                Origin::signed(account_key(TEST_SENDER)),
                TEST_PRODUCT_ID.as_bytes().to_owned(),
                account_key(TEST_ORGANIZATION),
                YEAR2,
                Some(vec![
                    ProductProperty::new(b"prop1", b"val1"),
                    ProductProperty::new(b"prop2", b"val2"),
                    ProductProperty::new(b"prop3", &LONG_VALUE.as_bytes().to_owned()),
                ])
            ),
            Error::<Test>::InvalidPropValue
        );
    })
}
