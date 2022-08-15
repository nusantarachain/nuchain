// This file is part of nuchain.

// Copyright (C) 2019-2022 Nusantara Chain Foundation.
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
use nuchain_runtime::{
	constants::currency::*, wasm_binary_unwrap, AuthorityDiscoveryConfig, BabeConfig,
	BalancesConfig, Block, CouncilConfig, DemocracyConfig, ElectionsConfig, GrandpaConfig,
	ImOnlineConfig, IndicesConfig, MaxNominations, NominationPoolsConfig, SessionConfig,
	SessionKeys, SocietyConfig, StakerStatus, StakingConfig, SudoConfig, SystemConfig,
	TechnicalCommitteeConfig,
};
use pallet_im_online::sr25519::AuthorityId as ImOnlineId;
use sc_chain_spec::ChainSpecExtension;
use sc_service::ChainType;
use sc_telemetry::TelemetryEndpoints;
use serde::{Deserialize, Serialize};
use sp_authority_discovery::AuthorityId as AuthorityDiscoveryId;
use sp_consensus_babe::AuthorityId as BabeId;
use sp_core::{crypto::UncheckedInto, sr25519, Pair, Public};
use sp_runtime::{
	traits::{IdentifyAccount, Verify},
	Perbill,
};

pub use nuchain_runtime::GenesisConfig;
pub use node_primitives::{AccountId, Balance, Signature};

type AccountPublic = <Signature as Verify>::Signer;

const STAGING_TELEMETRY_URL: &str = "wss://telemetry.nuchain.network/submit";

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
	/// The light sync state extension used by the sync-state rpc.
	pub light_sync_state: sc_sync_state_rpc::LightSyncStateExtension,
}

/// Specialized `ChainSpec`.
pub type ChainSpec = sc_service::GenericChainSpec<GenesisConfig, Extensions>;
/// Flaming Fir testnet generator
pub fn flaming_fir_config() -> Result<ChainSpec, String> {
	ChainSpec::from_json_bytes(&include_bytes!("../res/flaming-fir.json")[..])
}

fn session_keys(
	grandpa: GrandpaId,
	babe: BabeId,
	im_online: ImOnlineId,
	authority_discovery: AuthorityDiscoveryId,
) -> SessionKeys {
	SessionKeys { grandpa, babe, im_online, authority_discovery }
}

fn staging_testnet_config_genesis() -> GenesisConfig {
	#[rustfmt::skip]
	// stash, controller, session-key
	// generated with secret:
	// for i in 1 2 3 4 ; do for j in stash controller; do subkey inspect "$secret"/fir/$j/$i; done; done
	//
	// and
	//
	// for i in 1 2 3 4 ; do for j in session; do subkey --ed25519 inspect "$secret"//fir//$j//$i; done; done

	let initial_authorities: Vec<(
		AccountId,
		AccountId,
		GrandpaId,
		BabeId,
		ImOnlineId,
		AuthorityDiscoveryId,
	)> = vec![
		(
			// 5Fbsd6WXDGiLTxunqeK5BATNiocfCqu9bS1yArVjCgeBLkVy
			hex!["9c7a2ee14e565db0c69f78c7b4cd839fbf52b607d867e9e9c5a79042898a0d12"].into(),
			// 5EnCiV7wSHeNhjW3FSUwiJNkcc2SBkPLn5Nj93FmbLtBjQUq
			hex!["781ead1e2fa9ccb74b44c19d29cb2a7a4b5be3972927ae98cd3877523976a276"].into(),
			// 5Fb9ayurnxnaXj56CjmyQLBiadfRCqUbL2VWNbbe1nZU6wiC
			hex!["9becad03e6dcac03cee07edebca5475314861492cdfc96a2144a67bbe9699332"]
				.unchecked_into(),
			// 5EZaeQ8djPcq9pheJUhgerXQZt9YaHnMJpiHMRhwQeinqUW8
			hex!["6e7e4eb42cbd2e0ab4cae8708ce5509580b8c04d11f6758dbf686d50fe9f9106"]
				.unchecked_into(),
			// 5EZaeQ8djPcq9pheJUhgerXQZt9YaHnMJpiHMRhwQeinqUW8
			hex!["6e7e4eb42cbd2e0ab4cae8708ce5509580b8c04d11f6758dbf686d50fe9f9106"]
				.unchecked_into(),
			// 5EZaeQ8djPcq9pheJUhgerXQZt9YaHnMJpiHMRhwQeinqUW8
			hex!["6e7e4eb42cbd2e0ab4cae8708ce5509580b8c04d11f6758dbf686d50fe9f9106"]
				.unchecked_into(),
		),
		(
			// 5ERawXCzCWkjVq3xz1W5KGNtVx2VdefvZ62Bw1FEuZW4Vny2
			hex!["68655684472b743e456907b398d3a44c113f189e56d1bbfd55e889e295dfde78"].into(),
			// 5Gc4vr42hH1uDZc93Nayk5G7i687bAQdHHc9unLuyeawHipF
			hex!["c8dc79e36b29395413399edaec3e20fcca7205fb19776ed8ddb25d6f427ec40e"].into(),
			// 5EockCXN6YkiNCDjpqqnbcqd4ad35nU4RmA1ikM4YeRN4WcE
			hex!["7932cff431e748892fa48e10c63c17d30f80ca42e4de3921e641249cd7fa3c2f"]
				.unchecked_into(),
			// 5DhLtiaQd1L1LU9jaNeeu9HJkP6eyg3BwXA7iNMzKm7qqruQ
			hex!["482dbd7297a39fa145c570552249c2ca9dd47e281f0c500c971b59c9dcdcd82e"]
				.unchecked_into(),
			// 5DhLtiaQd1L1LU9jaNeeu9HJkP6eyg3BwXA7iNMzKm7qqruQ
			hex!["482dbd7297a39fa145c570552249c2ca9dd47e281f0c500c971b59c9dcdcd82e"]
				.unchecked_into(),
			// 5DhLtiaQd1L1LU9jaNeeu9HJkP6eyg3BwXA7iNMzKm7qqruQ
			hex!["482dbd7297a39fa145c570552249c2ca9dd47e281f0c500c971b59c9dcdcd82e"]
				.unchecked_into(),
		),
		(
			// 5DyVtKWPidondEu8iHZgi6Ffv9yrJJ1NDNLom3X9cTDi98qp
			hex!["547ff0ab649283a7ae01dbc2eb73932eba2fb09075e9485ff369082a2ff38d65"].into(),
			// 5FeD54vGVNpFX3PndHPXJ2MDakc462vBCD5mgtWRnWYCpZU9
			hex!["9e42241d7cd91d001773b0b616d523dd80e13c6c2cab860b1234ef1b9ffc1526"].into(),
			// 5E1jLYfLdUQKrFrtqoKgFrRvxM3oQPMbf6DfcsrugZZ5Bn8d
			hex!["5633b70b80a6c8bb16270f82cca6d56b27ed7b76c8fd5af2986a25a4788ce440"]
				.unchecked_into(),
			// 5DhKqkHRkndJu8vq7pi2Q5S3DfftWJHGxbEUNH43b46qNspH
			hex!["482a3389a6cf42d8ed83888cfd920fec738ea30f97e44699ada7323f08c3380a"]
				.unchecked_into(),
			// 5DhKqkHRkndJu8vq7pi2Q5S3DfftWJHGxbEUNH43b46qNspH
			hex!["482a3389a6cf42d8ed83888cfd920fec738ea30f97e44699ada7323f08c3380a"]
				.unchecked_into(),
			// 5DhKqkHRkndJu8vq7pi2Q5S3DfftWJHGxbEUNH43b46qNspH
			hex!["482a3389a6cf42d8ed83888cfd920fec738ea30f97e44699ada7323f08c3380a"]
				.unchecked_into(),
		),
		(
			// 5HYZnKWe5FVZQ33ZRJK1rG3WaLMztxWrrNDb1JRwaHHVWyP9
			hex!["f26cdb14b5aec7b2789fd5ca80f979cef3761897ae1f37ffb3e154cbcc1c2663"].into(),
			// 5EPQdAQ39WQNLCRjWsCk5jErsCitHiY5ZmjfWzzbXDoAoYbn
			hex!["66bc1e5d275da50b72b15de072a2468a5ad414919ca9054d2695767cf650012f"].into(),
			// 5DMa31Hd5u1dwoRKgC4uvqyrdK45RHv3CpwvpUC1EzuwDit4
			hex!["3919132b851ef0fd2dae42a7e734fe547af5a6b809006100f48944d7fae8e8ef"]
				.unchecked_into(),
			// 5C4vDQxA8LTck2xJEy4Yg1hM9qjDt4LvTQaMo4Y8ne43aU6x
			hex!["00299981a2b92f878baaf5dbeba5c18d4e70f2a1fcd9c61b32ea18daf38f4378"]
				.unchecked_into(),
			// 5C4vDQxA8LTck2xJEy4Yg1hM9qjDt4LvTQaMo4Y8ne43aU6x
			hex!["00299981a2b92f878baaf5dbeba5c18d4e70f2a1fcd9c61b32ea18daf38f4378"]
				.unchecked_into(),
			// 5C4vDQxA8LTck2xJEy4Yg1hM9qjDt4LvTQaMo4Y8ne43aU6x
			hex!["00299981a2b92f878baaf5dbeba5c18d4e70f2a1fcd9c61b32ea18daf38f4378"]
				.unchecked_into(),
		),
	];

	// generated with secret: subkey inspect "$secret"/fir
	let root_key: AccountId = hex![
		// 5Ff3iXP75ruzroPWRP2FYBHWnmGGBSb63857BgnzCoXNxfPo
		"9ee5e5bdc0ec239eb164f865ecc345ce4c88e76ee002e0f7e318097347471809"
	]
	.into();

	let endowed_accounts: Vec<AccountId> = vec![root_key.clone()];

	build_genesis(initial_authorities, vec![], root_key, Some(endowed_accounts))
}

/// Staging testnet config.
pub fn staging_testnet_config() -> ChainSpec {
	let boot_nodes = vec![];
	ChainSpec::from_genesis(
		"Staging Testnet",
		"staging_testnet",
		ChainType::Live,
		staging_testnet_config_genesis,
		boot_nodes,
		Some(
			TelemetryEndpoints::new(vec![(STAGING_TELEMETRY_URL.to_string(), 0)])
				.expect("Staging telemetry url is valid; qed"),
		),
		None,
		None,
		None,
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
) -> (AccountId, AccountId, GrandpaId, BabeId, ImOnlineId, AuthorityDiscoveryId) {
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
	initial_nominators: Vec<AccountId>,
	root_key: AccountId,
	endowed_accounts: Option<Vec<AccountId>>,
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
	// endow all authorities and nominators.
	initial_authorities
		.iter()
		.map(|x| &x.0)
		.chain(initial_nominators.iter())
		.for_each(|x| {
			if !endowed_accounts.contains(x) {
				endowed_accounts.push(x.clone())
			}
		});

	// stakers: all validators and nominators.
	let mut rng = rand::thread_rng();
	let stakers = initial_authorities
		.iter()
		.map(|x| (x.0.clone(), x.1.clone(), STASH, StakerStatus::Validator))
		.chain(initial_nominators.iter().map(|x| {
			use rand::{seq::SliceRandom, Rng};
			let limit = (MaxNominations::get() as usize).min(initial_authorities.len());
			let count = rng.gen::<usize>() % limit;
			let nominations = initial_authorities
				.as_slice()
				.choose_multiple(&mut rng, count)
				.into_iter()
				.map(|choice| choice.0.clone())
				.collect::<Vec<_>>();
			(x.clone(), x.clone(), STASH, StakerStatus::Nominator(nominations))
		}))
		.collect::<Vec<_>>();

	let num_endowed_accounts = endowed_accounts.len();

	const ENDOWMENT: Balance = 100_000 * DOLLARS;
	const STASH: Balance = ENDOWMENT / 1000;

	GenesisConfig {
		system: SystemConfig { code: wasm_binary_unwrap().to_vec() },
		balances: BalancesConfig {
			balances: endowed_accounts.iter().cloned().map(|x| (x, ENDOWMENT)).collect(),
		},
		indices: IndicesConfig { indices: vec![] },
		session: SessionConfig {
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
		},
		staking: StakingConfig {
			validator_count: initial_authorities.len() as u32,
			minimum_validator_count: initial_authorities.len() as u32,
			invulnerables: initial_authorities.iter().map(|x| x.0.clone()).collect(),
			slash_reward_fraction: Perbill::from_percent(10),
			stakers,
			..Default::default()
		},
		democracy: DemocracyConfig::default(),
		elections: ElectionsConfig {
			members: endowed_accounts
				.iter()
				.take((num_endowed_accounts + 1) / 2)
				.cloned()
				.map(|member| (member, STASH))
				.collect(),
		},
		council: CouncilConfig::default(),
		technical_committee: TechnicalCommitteeConfig {
			members: endowed_accounts
				.iter()
				.take((num_endowed_accounts + 1) / 2)
				.cloned()
				.collect(),
			phantom: Default::default(),
		},
		sudo: SudoConfig { key: Some(root_key) },
		babe: BabeConfig {
			authorities: vec![],
			epoch_config: Some(nuchain_runtime::BABE_GENESIS_EPOCH_CONFIG),
		},
		im_online: ImOnlineConfig { keys: vec![] },
		authority_discovery: AuthorityDiscoveryConfig { keys: vec![] },
		grandpa: GrandpaConfig { authorities: vec![] },
		technical_membership: Default::default(),
		treasury: Default::default(),
		society: SocietyConfig {
			members: endowed_accounts
				.iter()
				.take((num_endowed_accounts + 1) / 2)
				.cloned()
				.collect(),
			pot: 0,
			max_members: 999,
		},
		vesting: Default::default(),
		assets: Default::default(),
		// gilt: Default::default(),
		transaction_storage: Default::default(),
		transaction_payment: Default::default(),
		did: Default::default(),
		organization: Default::default(),
		certificate: Default::default(),
        liquidity: Default::default(),
		// alliance: Default::default(),
		// alliance_motion: Default::default(),
		nomination_pools: NominationPoolsConfig {
			min_create_bond: 10 * DOLLARS,
			min_join_bond: 1 * DOLLARS,
			..Default::default()
		},
	}
}

pub fn testnet_genesis(	initial_authorities: Vec<(
    AccountId,
    AccountId,
    GrandpaId,
    BabeId,
    ImOnlineId,
    AuthorityDiscoveryId,
)>,
initial_nominators: Vec<AccountId>,
root_key: AccountId,
endowed_accounts: Option<Vec<AccountId>>) -> GenesisConfig {
    build_genesis(initial_authorities, initial_nominators, root_key, endowed_accounts,)
}

fn development_config_genesis() -> GenesisConfig {
	build_genesis(
		vec![authority_keys_from_seed("Alice")],
		vec![],
		get_account_id_from_seed::<sr25519::Public>("Alice"),
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
		None,
		Default::default(),
	)
}

fn local_testnet_genesis() -> GenesisConfig {
	build_genesis(
		vec![authority_keys_from_seed("Alice"), authority_keys_from_seed("Bob")],
		vec![],
		get_account_id_from_seed::<sr25519::Public>("Alice"),
		None,
	)
}

/// Local testnet config (multivalidator Alice + Bob)
pub fn local_testnet_config() -> ChainSpec {
	ChainSpec::from_genesis(
		"Local Testnet",
		"local_testnet",
		ChainType::Local,
		local_testnet_genesis,
		vec![],
		None,
		None,
		None,
		None,
		Default::default(),
	)
}

/// Load main config
pub fn main_config() -> Result<ChainSpec, String> {
	ChainSpec::from_json_bytes(&include_bytes!("../res/nuchain.json")[..])
}

/// Load gama network config
pub fn gama_config() -> Result<ChainSpec, String> {
	ChainSpec::from_json_bytes(&include_bytes!("../res/nuchain-gama.json")[..])
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
		vec![],
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
        // false,
        // Some(100_000 * DOLLARS),
    )
}

use sc_chain_spec::Properties;

/// Production configuration
pub fn prod_config() -> ChainSpec {
    let boot_nodes = vec![];
    // let properties = serde_json::from_str(
    //     r#"{
    //         "ss58Format": 99,
    //         "tokenDecimals": 10,
    //         "tokenSymbol": "ARA"
    //     }"#,
    // )
    // .unwrap();
	let mut properties = Properties::new();
	properties.insert("ss58Format".into(), 99.into());
	properties.insert("tokenDecimals".into(), 10.into());
	properties.insert("tokenSymbol".into(), "ARA".into());

    ChainSpec::from_genesis(
        "Nuchain Gama",
        "gama",
        ChainType::Live,
        prod_genesis,
        boot_nodes,
        Some(
            TelemetryEndpoints::new(vec![(STAGING_TELEMETRY_URL.to_string(), 0)])
                .expect("Staging telemetry url is valid; qed"),
        ),
        None,
		None,
        Some(properties),
        Default::default(),
    )
}

#[cfg(test)]
pub(crate) mod tests {
	use super::*;
	use crate::service::{new_full_base, NewFullBase};
	use sc_service_test;
	use sp_runtime::BuildStorage;

	fn local_testnet_genesis_instant_single() -> GenesisConfig {
		build_genesis(
			vec![authority_keys_from_seed("Alice")],
			vec![],
			get_account_id_from_seed::<sr25519::Public>("Alice"),
			None,
		)
	}

	/// Local testnet config (single validator - Alice)
	pub fn integration_test_config_with_single_authority() -> ChainSpec {
		ChainSpec::from_genesis(
			"Integration Test",
			"test",
			ChainType::Development,
			local_testnet_genesis_instant_single,
			vec![],
			None,
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
			local_testnet_genesis,
			vec![],
			None,
			None,
			None,
			None,
			Default::default(),
		)
	}

	#[test]
	#[ignore]
	fn test_connectivity() {
		sp_tracing::try_init_simple();

		sc_service_test::connectivity(integration_test_config_with_two_authorities(), |config| {
			let NewFullBase { task_manager, client, network, transaction_pool, .. } =
				new_full_base(config, false, |_, _| ())?;
			Ok(sc_service_test::TestNetComponents::new(
				task_manager,
				client,
				network,
				transaction_pool,
			))
		});
	}

	#[test]
	fn test_create_development_chain_spec() {
		development_config().build_storage().unwrap();
	}

	#[test]
	fn test_create_local_testnet_chain_spec() {
		local_testnet_config().build_storage().unwrap();
	}

	#[test]
	fn test_staging_test_net_chain_spec() {
		staging_testnet_config().build_storage().unwrap();
	}
}
