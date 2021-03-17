// This file is part of Nuchain.

// Copyright (C) 2021 Nuchain
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

//! Substrate chain configurations.

use grandpa_primitives::AuthorityId as GrandpaId;
use hex_literal::hex;
use nuchain_runtime::constants::currency::*;
use nuchain_runtime::Block;
use nuchain_runtime::{
    wasm_binary_unwrap, AuthorityDiscoveryConfig, BabeConfig, BalancesConfig, ContractsConfig,
    CouncilConfig, DemocracyConfig, ElectionsConfig, GrandpaConfig, ImOnlineConfig, IndicesConfig,
    SessionConfig, SessionKeys, SocietyConfig, StakerStatus, StakingConfig, SudoConfig,
    SystemConfig, TechnicalCommitteeConfig,
};
use pallet_im_online::sr25519::AuthorityId as ImOnlineId;
use sc_chain_spec::ChainSpecExtension;
use sc_service::ChainType;
use sc_telemetry::TelemetryEndpoints;
use serde::{Deserialize, Serialize};
use serde_json;
use sp_authority_discovery::AuthorityId as AuthorityDiscoveryId;
use sp_consensus_babe::AuthorityId as BabeId;
use sp_core::{crypto::UncheckedInto, sr25519, Pair, Public};
use sp_runtime::{
    traits::{IdentifyAccount, Verify},
    Perbill,
};

pub use node_primitives::{AccountId, Balance, Signature};
pub use nuchain_runtime::GenesisConfig;

type AccountPublic = <Signature as Verify>::Signer;

const STAGING_TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";

/// Node `ChainSpec` extensions.
///
/// Additional parameters for some Substrate core modules,
/// customizable from the chain spec.
#[derive(Default, Clone, Serialize, Deserialize, ChainSpecExtension)]
#[serde(rename_all = "camelCase")]
pub struct Extensions {
    /// Block numbers with known hashes.
    pub fork_blocks: sc_client_api::ForkBlocks<Block>,
    /// Known bad block hashes.
    pub bad_blocks: sc_client_api::BadBlocks<Block>,
}

/// Specialized `ChainSpec`.
pub type ChainSpec = sc_service::GenericChainSpec<GenesisConfig, Extensions>;
/// Flaming Fir testnet generator
pub fn flaming_fir_config() -> Result<ChainSpec, String> {
    ChainSpec::from_json_bytes(&include_bytes!("../res/flaming-fir.json")[..])
}

/// Main config
pub fn main_config() -> Result<ChainSpec, String> {
    ChainSpec::from_json_bytes(&include_bytes!("../res/nuchain.json")[..])
}

/// Testnet config
pub fn testnet_config() -> Result<ChainSpec, String> {
    ChainSpec::from_json_bytes(&include_bytes!("../res/testnet.json")[..])
}

fn session_keys(
    grandpa: GrandpaId,
    babe: BabeId,
    im_online: ImOnlineId,
    authority_discovery: AuthorityDiscoveryId,
) -> SessionKeys {
    SessionKeys {
        grandpa,
        babe,
        im_online,
        authority_discovery,
    }
}

fn staging_config_genesis() -> GenesisConfig {
    // stash, controller, session-key
    // generated with secret:
    // for i in 1 2 3 4 ; do for j in stash controller; do subkey inspect "$secret"/fir/$j/$i; done; done
    // and
    // for i in 1 2 3 4 ; do for j in session; do subkey --ed25519 inspect "$secret"//fir//$j//$i; done; done

    let initial_authorities: Vec<(
        AccountId,
        AccountId,
        GrandpaId,
        BabeId,
        ImOnlineId,
        AuthorityDiscoveryId,
    )> = vec![
        // Stash AccountId (sr25519)
        // Controller AccountId (sr25519)
        // GradpadId (ed25519)
        // BabeId (sr25519) / babe
        // ImOnlineId (sr25519) / imon
        // AuthorityDiscovery (sr25519) / audi
        //----------------------------------------------------------------
        (
            // 5FxKovft7pM663rr4Smtbj4CZzt82TaykWFZP2H4rjCNTiJu
            hex!["ac133e5ced8c63f4028be2f9f10da8b5d1f9d270ba03820723361da981a5dc18"].into(),
            // 5HMsJCtxzvVHa458CxsVsuboP1Nee6sE7KjhfxbDXCP5j3aM
            hex!["ea441e35c86bac239d3e40bb6ff0ed9008447d02d90e20c3044e06e301297965"].into(),
            // GranpaId: 5F4wPxMnFGNGi5docWuMx7G7BfdKEx5wTiiDP3MFByACmNfR
            hex!["84e24732c91231c3210fa6f2f3b9b777a92f61d5d1fede6f43c78620abfe855d"]
                .unchecked_into(),
            //---- SESSIONS ----
            // 5Ca9DuynzqbXFUQZuEkuhVVZS7abaZQL2dADJ8U5oz4cXjxR
            hex!["167381df0eec9c3fd442d130188150100ad11d00a9ca66e3d425409b1e083f3c"]
                .unchecked_into(),
            hex!["167381df0eec9c3fd442d130188150100ad11d00a9ca66e3d425409b1e083f3c"]
                .unchecked_into(),
            hex!["167381df0eec9c3fd442d130188150100ad11d00a9ca66e3d425409b1e083f3c"]
                .unchecked_into(),
        ),
        (
            // 5HGZ4bYs6dNBkv5FDm8vDnX6Dmu9BKMu5hnp4VQEvqpxKCmk
            hex!["e63681f88b055258860b53f3e87c959c3da95d6b77becacb2fc5afcef021742e"].into(),
            // 5GVmNGawT1CPERzsRGPXRqRAE94HBmY3mhJLsoMMMzdMc2CF
            hex!["c40de2c66879a462f477c706db7aeb83b67f4076be7f6bdfd74f002afcf6e22e"].into(),
            // GrandpaId: 5Cf1ayVSoxQ39XV44BWBVTjF4SSQE2CoTsxLR26gnkDpokFG
            hex!["1a2a06ba1f03b6fa2591da9005f100053b24225f5231abce6d1547704ff740e9"]
                .unchecked_into(),
            //---- SESSIONS ----
            // 5H6AKvZeTDkvZKVWxyqzGjgj4NezwomVYEi6KcjtsZN7dM8F
            hex!["de4984b4344a796f989b34ab234adc64b6af022f069e33657937ca68665c547c"]
                .unchecked_into(),
            hex!["de4984b4344a796f989b34ab234adc64b6af022f069e33657937ca68665c547c"]
                .unchecked_into(),
            hex!["de4984b4344a796f989b34ab234adc64b6af022f069e33657937ca68665c547c"]
                .unchecked_into(),
        ),
    ];

    let root_key: AccountId = hex![
        "18bff030bef78621b59562a9633d6c8ec358a96c070358de3fcd7fd8d2879e35"
    ]
    .into();

    build_genesis(
        initial_authorities,
        root_key.clone(),
        Some(vec![
            root_key,
            // reserved authorities
            hex!["3af749c23d1c17bc0c822363b3e2620d6f473cb5e9631d10449bdb0dea683130"].into(),
            hex!["ee735365ca9e1bdebe0b7fbb7e781ff88a63d8e7c60569a399d256497d618813"].into(),
            hex!["4a8f386d7b8849e2be3a67a2182fefee87138b4b908e00e7386516a4f82bb576"].into(),
            hex!["ee7c3224fe1d012e0c5cdf1eb1b1c6164752dff43bb8f0ca95e8521a6ed3a37a"].into(),
            // for authority validators
            hex!["c83104c7eba84373392336d71ef4915b7a45c4966d1dbc82eee146109b390e5f"].into(),
            hex!["6671d91c741357a54eb81176d74bbf42445d4883b90148179a8b49aaa459b51e"].into(),
        ]),
        false,
        None,
    )
}

/// Configuration for testnet
pub fn staging_config() -> ChainSpec {
    let boot_nodes = vec![];
    let properties = serde_json::from_str(
        r#"{
            "tokenDecimals": 10,
            "tokenSymbol": "ARA"
        }"#,
    )
    .unwrap();
    ChainSpec::from_genesis(
        "Nuchain Testnet",
        "nuchain_testnet",
        ChainType::Live,
        staging_config_genesis,
        boot_nodes,
        Some(
            TelemetryEndpoints::new(vec![(STAGING_TELEMETRY_URL.to_string(), 0)])
                .expect("Staging telemetry url is valid; qed"),
        ),
        Some("nuct"),
        Some(properties),
        Default::default(),
    )
}

/// Helper function to generate a crypto pair from seed
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
    TPublic::Pair::from_string(&format!("//{}", seed), None)
        .expect("static values are valid; qed")
        .public()
}

/// Helper function to generate an account ID from seed
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
where
    AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
    AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

/// Helper function to generate stash, controller and session key from seed
pub fn authority_keys_from_seed(
    seed: &str,
) -> (
    AccountId,
    AccountId,
    GrandpaId,
    BabeId,
    ImOnlineId,
    AuthorityDiscoveryId,
) {
    (
        get_account_id_from_seed::<sr25519::Public>(&format!("{}//stash", seed)),
        get_account_id_from_seed::<sr25519::Public>(seed),
        get_from_seed::<GrandpaId>(seed),
        get_from_seed::<BabeId>(seed),
        get_from_seed::<ImOnlineId>(seed),
        get_from_seed::<AuthorityDiscoveryId>(seed),
    )
}

/// Helper function to create GenesisConfig for testing
pub fn build_genesis(
    initial_authorities: Vec<(
        AccountId,
        AccountId,
        GrandpaId,
        BabeId,
        ImOnlineId,
        AuthorityDiscoveryId,
    )>,
    root_key: AccountId,
    endowed_accounts: Option<Vec<AccountId>>,
    enable_println: bool,
    endowment_balance: Option<Balance>,
) -> GenesisConfig {
    let mut endowed_accounts: Vec<AccountId> = endowed_accounts.unwrap_or_else(|| {
        vec![
            get_account_id_from_seed::<sr25519::Public>("Alice"),
            get_account_id_from_seed::<sr25519::Public>("Bob"),
            get_account_id_from_seed::<sr25519::Public>("Charlie"),
            get_account_id_from_seed::<sr25519::Public>("Dave"),
            get_account_id_from_seed::<sr25519::Public>("Eve"),
            get_account_id_from_seed::<sr25519::Public>("Ferdie"),
            get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
            get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
            get_account_id_from_seed::<sr25519::Public>("Charlie//stash"),
            get_account_id_from_seed::<sr25519::Public>("Dave//stash"),
            get_account_id_from_seed::<sr25519::Public>("Eve//stash"),
            get_account_id_from_seed::<sr25519::Public>("Ferdie//stash"),
        ]
    });
    initial_authorities.iter().for_each(|x| {
        if !endowed_accounts.contains(&x.0) {
            endowed_accounts.push(x.0.clone())
        }
    });

    let num_endowed_accounts = endowed_accounts.len();

    let endowment: Balance = endowment_balance.unwrap_or_else(|| 1_000_000 * DOLLARS);
    let stash: Balance = endowment / 100;

    GenesisConfig {
        frame_system: Some(SystemConfig {
            code: wasm_binary_unwrap().to_vec(),
            changes_trie_config: Default::default(),
        }),
        pallet_balances: Some(BalancesConfig {
            balances: endowed_accounts
                .iter()
                .cloned()
                .map(|x| {
                    if x == root_key {
                        (x, endowment * 9)
                    } else {
                        (x, endowment)
                    }
                })
                .collect(),
        }),
        pallet_indices: Some(IndicesConfig { indices: vec![] }),
        pallet_session: Some(SessionConfig {
            keys: initial_authorities
                .iter()
                .map(|x| {
                    (
                        x.0.clone(),
                        x.0.clone(),
                        session_keys(x.2.clone(), x.3.clone(), x.4.clone(), x.5.clone()),
                    )
                })
                .collect::<Vec<_>>(),
        }),
        pallet_staking: Some(StakingConfig {
            validator_count: initial_authorities.len() as u32 * 2,
            minimum_validator_count: initial_authorities.len() as u32,
            stakers: initial_authorities
                .iter()
                .map(|x| (x.0.clone(), x.1.clone(), stash, StakerStatus::Validator))
                .collect(),
            invulnerables: initial_authorities.iter().map(|x| x.0.clone()).collect(),
            slash_reward_fraction: Perbill::from_percent(10),
            ..Default::default()
        }),
        pallet_democracy: Some(DemocracyConfig::default()),
        pallet_elections_phragmen: Some(ElectionsConfig {
            members: endowed_accounts
                .iter()
                .take((num_endowed_accounts + 1) / 2)
                .cloned()
                .map(|member| (member, stash))
                .collect(),
        }),
        pallet_collective_Instance1: Some(CouncilConfig::default()),
        pallet_collective_Instance2: Some(TechnicalCommitteeConfig {
            members: endowed_accounts
                .iter()
                .take((num_endowed_accounts + 1) / 2)
                .cloned()
                .collect(),
            phantom: Default::default(),
        }),
        pallet_contracts: Some(ContractsConfig {
            current_schedule: pallet_contracts::Schedule {
                enable_println, // this should only be enabled on development chains
                ..Default::default()
            },
        }),
        pallet_sudo: Some(SudoConfig { key: root_key }),
        pallet_babe: Some(BabeConfig {
            authorities: vec![],
        }),
        pallet_im_online: Some(ImOnlineConfig { keys: vec![] }),
        pallet_authority_discovery: Some(AuthorityDiscoveryConfig { keys: vec![] }),
        pallet_grandpa: Some(GrandpaConfig {
            authorities: vec![],
        }),
        pallet_membership_Instance1: Some(Default::default()),
        pallet_treasury: Some(Default::default()),
        pallet_society: Some(SocietyConfig {
            members: endowed_accounts
                .iter()
                .take((num_endowed_accounts + 1) / 2)
                .cloned()
                .collect(),
            pot: 0,
            max_members: 999,
        }),
        pallet_vesting: Some(Default::default()),
    }
}

fn development_config_genesis() -> GenesisConfig {
    build_genesis(
        vec![authority_keys_from_seed("Alice")],
        get_account_id_from_seed::<sr25519::Public>("Alice"),
        None,
        true,
        None,
    )
}

/// Development config (single validator Alice)
pub fn development_config() -> ChainSpec {
    ChainSpec::from_genesis(
        "Development",
        "dev",
        ChainType::Development,
        development_config_genesis,
        vec![],
        None,
        None,
        None,
        Default::default(),
    )
}

fn local_build_genesis() -> GenesisConfig {
    build_genesis(
        vec![
            authority_keys_from_seed("Alice"),
            authority_keys_from_seed("Bob"),
        ],
        get_account_id_from_seed::<sr25519::Public>("Alice"),
        None,
        false,
        None,
    )
}

/// Local testnet config (multivalidator Alice + Bob)
pub fn local_staging_config() -> ChainSpec {
    ChainSpec::from_genesis(
        "Local Testnet",
        "local_testnet",
        ChainType::Local,
        local_build_genesis,
        vec![],
        None,
        None,
        None,
        Default::default(),
    )
}

/// Production genesis
fn prod_genesis() -> GenesisConfig {
    let sudo_acc: AccountId =
        hex!["18bff030bef78621b59562a9633d6c8ec358a96c070358de3fcd7fd8d2879e35"].into();
    let authorities: Vec<(
        AccountId,
        AccountId,
        GrandpaId,
        BabeId,
        ImOnlineId,
        AuthorityDiscoveryId,
    )> = vec![
        // Stash AccountId (sr25519)
        // Controller AccountId (sr25519)
        // GradpadId (ed25519)
        // BabeId (sr25519) / babe
        // ImOnlineId (sr25519) / imon
        // AuthorityDiscovery (sr25519) / audi
        //----------------------------------------------------------------
        (
            // 5FxKovft7pM663rr4Smtbj4CZzt82TaykWFZP2H4rjCNTiJu
            hex!["ac133e5ced8c63f4028be2f9f10da8b5d1f9d270ba03820723361da981a5dc18"].into(),
            // 5HMsJCtxzvVHa458CxsVsuboP1Nee6sE7KjhfxbDXCP5j3aM
            hex!["ea441e35c86bac239d3e40bb6ff0ed9008447d02d90e20c3044e06e301297965"].into(),
            // GranpaId: 5F4wPxMnFGNGi5docWuMx7G7BfdKEx5wTiiDP3MFByACmNfR
            hex!["84e24732c91231c3210fa6f2f3b9b777a92f61d5d1fede6f43c78620abfe855d"]
                .unchecked_into(),
            //---- SESSIONS ----
            // 5Ca9DuynzqbXFUQZuEkuhVVZS7abaZQL2dADJ8U5oz4cXjxR
            hex!["167381df0eec9c3fd442d130188150100ad11d00a9ca66e3d425409b1e083f3c"]
                .unchecked_into(),
            hex!["167381df0eec9c3fd442d130188150100ad11d00a9ca66e3d425409b1e083f3c"]
                .unchecked_into(),
            hex!["167381df0eec9c3fd442d130188150100ad11d00a9ca66e3d425409b1e083f3c"]
                .unchecked_into(),
        ),
        (
            // 5HGZ4bYs6dNBkv5FDm8vDnX6Dmu9BKMu5hnp4VQEvqpxKCmk
            hex!["e63681f88b055258860b53f3e87c959c3da95d6b77becacb2fc5afcef021742e"].into(),
            // 5GVmNGawT1CPERzsRGPXRqRAE94HBmY3mhJLsoMMMzdMc2CF
            hex!["c40de2c66879a462f477c706db7aeb83b67f4076be7f6bdfd74f002afcf6e22e"].into(),
            // GrandpaId: 5Cf1ayVSoxQ39XV44BWBVTjF4SSQE2CoTsxLR26gnkDpokFG
            hex!["1a2a06ba1f03b6fa2591da9005f100053b24225f5231abce6d1547704ff740e9"]
                .unchecked_into(),
            //---- SESSIONS ----
            // 5H6AKvZeTDkvZKVWxyqzGjgj4NezwomVYEi6KcjtsZN7dM8F
            hex!["de4984b4344a796f989b34ab234adc64b6af022f069e33657937ca68665c547c"]
                .unchecked_into(),
            hex!["de4984b4344a796f989b34ab234adc64b6af022f069e33657937ca68665c547c"]
                .unchecked_into(),
            hex!["de4984b4344a796f989b34ab234adc64b6af022f069e33657937ca68665c547c"]
                .unchecked_into(),
        ),
    ];
    build_genesis(
        authorities,
        sudo_acc.clone(),
        Some(vec![
            sudo_acc,
            // reserved authorities
            hex!["3af749c23d1c17bc0c822363b3e2620d6f473cb5e9631d10449bdb0dea683130"].into(),
            hex!["ee735365ca9e1bdebe0b7fbb7e781ff88a63d8e7c60569a399d256497d618813"].into(),
            hex!["4a8f386d7b8849e2be3a67a2182fefee87138b4b908e00e7386516a4f82bb576"].into(),
            hex!["ee7c3224fe1d012e0c5cdf1eb1b1c6164752dff43bb8f0ca95e8521a6ed3a37a"].into(),
            // for authority validators
            hex!["c83104c7eba84373392336d71ef4915b7a45c4966d1dbc82eee146109b390e5f"].into(),
            hex!["6671d91c741357a54eb81176d74bbf42445d4883b90148179a8b49aaa459b51e"].into(),
        ]),
        false,
        Some(100_000 * DOLLARS),
    )
}

/// Production configuration
pub fn prod_config() -> ChainSpec {
    // ChainSpec::from_json_bytes(&include_bytes!("../res/prod.json"))
    let boot_nodes = vec![];
    let properties = serde_json::from_str(
        r#"{
            "ss58Format": 99,
            "tokenDecimals": 10,
            "tokenSymbol": "ARA"
        }"#,
    )
    .unwrap();
    ChainSpec::from_genesis(
        "Nuchain",
        "nuc01", // fase1
        ChainType::Live,
        prod_genesis,
        boot_nodes,
        Some(
            TelemetryEndpoints::new(vec![(STAGING_TELEMETRY_URL.to_string(), 0)])
                .expect("Staging telemetry url is valid; qed"),
        ),
        Some("nuc"),
        Some(properties),
        Default::default(),
    )
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::service::{new_full_base, new_light_base, NewFullBase};
    use sc_service_test;
    use sp_runtime::BuildStorage;

    fn local_build_genesis_instant_single() -> GenesisConfig {
        build_genesis(
            vec![authority_keys_from_seed("Alice")],
            get_account_id_from_seed::<sr25519::Public>("Alice"),
            None,
            false,
        )
    }

    /// Local testnet config (single validator - Alice)
    pub fn integration_test_config_with_single_authority() -> ChainSpec {
        ChainSpec::from_genesis(
            "Integration Test",
            "test",
            ChainType::Development,
            local_build_genesis_instant_single,
            vec![],
            None,
            None,
            None,
            Default::default(),
        )
    }

    /// Local testnet config (multivalidator Alice + Bob)
    pub fn integration_test_config_with_two_authorities() -> ChainSpec {
        ChainSpec::from_genesis(
            "Integration Test",
            "test",
            ChainType::Development,
            local_build_genesis,
            vec![],
            None,
            None,
            None,
            Default::default(),
        )
    }

    #[test]
    #[ignore]
    fn test_connectivity() {
        sc_service_test::connectivity(
            integration_test_config_with_two_authorities(),
            |config| {
                let NewFullBase {
                    task_manager,
                    client,
                    network,
                    transaction_pool,
                    ..
                } = new_full_base(config, |_, _| ())?;
                Ok(sc_service_test::TestNetComponents::new(
                    task_manager,
                    client,
                    network,
                    transaction_pool,
                ))
            },
            |config| {
                let (keep_alive, _, _, client, network, transaction_pool) = new_light_base(config)?;
                Ok(sc_service_test::TestNetComponents::new(
                    keep_alive,
                    client,
                    network,
                    transaction_pool,
                ))
            },
        );
    }

    #[test]
    fn test_create_development_chain_spec() {
        development_config().build_storage().unwrap();
    }

    #[test]
    fn test_create_local_testnet_chain_spec() {
        local_staging_config().build_storage().unwrap();
    }

    #[test]
    fn test_staging_test_net_chain_spec() {
        staging_staging_config().build_storage().unwrap();
    }

    #[test]
    fn test_create_prod_chain_spec() {
        prod_config().build_storage().unwrap();
    }
}
