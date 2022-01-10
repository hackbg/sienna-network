use std::convert::{TryFrom, TryInto};

use lend_shared::{
    impl_contract_storage,
    impl_contract_storage_option,
    fadroma::{
        schemars,
        schemars::JsonSchema,
        uint256::Uint256,
        cosmwasm_std::{
            HumanAddr, CanonicalAddr, Extern,
            StdResult, Api, Storage, Querier,
            StdError, Binary, Order
        },
        cosmwasm_storage::{Bucket, ReadonlyBucket},
        crypto::sha_256,
        storage::{load, save, ns_load, ns_save, IterableStorage},
        Canonize, Humanize,
        ContractLink, Decimal256
    },
    interfaces::overseer::{Pagination, Market}
};
use serde::{Deserialize, Serialize};

const PAGINATION_LIMIT: u8 = 30;

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Constants {
    pub close_factor: Decimal256,
    pub premium: Decimal256
}

pub struct Contracts;
pub struct Markets;

#[derive(Clone)]
pub struct Borrower {
    id: BorrowerId
}

#[derive(PartialEq, Clone, Debug)]
pub struct BorrowerId([u8; 32]);

impl Constants {
    const KEY: &'static [u8] = b"constants";

    #[inline]
    pub fn load(storage: &impl Storage) -> StdResult<Self> {
        Ok(load(storage, Self::KEY)?.unwrap())
    }

    #[inline]
    pub fn save(
        &self,
        storage: &mut impl Storage
    ) -> StdResult<()> {
        save(storage, Self::KEY, self)
    }
}

impl Contracts {
    impl_contract_storage!(save_oracle, load_oracle, b"oracle");
    impl_contract_storage!(save_self_ref, load_self_ref, b"self");
}

impl Markets {
    const NS: &'static [u8] = b"markets";

    pub fn push<S: Storage, A: Api, Q: Querier>(
        deps: &mut Extern<S, A, Q>,
        market: &Market<HumanAddr>
    ) -> StdResult<()> {
        let market = market.canonize(&deps.api)?;

        if ns_load::<Option<u64>, _>(
            &deps.storage,
            Self::NS,
            market.contract.address.as_slice()
        )?.is_some() {
            return Err(StdError::generic_err("Token is already registered as collateral."));
        }

        let index = IterableStorage::new(Self::NS)
            .push(&mut deps.storage, &market)?;

        ns_save(
            &mut deps.storage,
            Self::NS,
            market.contract.address.as_slice(),
            &index
        )
    }

    pub fn get_by_addr<S: Storage, A: Api, Q: Querier>(
        deps: &Extern<S, A, Q>,
        market: &HumanAddr
    ) -> StdResult<Market<HumanAddr>> {
        let result = Self::load(
            &deps.storage,
            Self::get_id(deps, market)?
        )?.unwrap();

        result.humanize(&deps.api)
    }

    pub fn get_by_id<S: Storage, A: Api, Q: Querier>(
        deps: &Extern<S, A, Q>,
        id: u64
    ) -> StdResult<Option<Market<HumanAddr>>> {
        let result = Self::load(&deps.storage, id)?;

        match result {
            Some(market) => Ok(Some(market.humanize(&deps.api)?)),
            None => Ok(None)
        }
    }

    pub fn get_id<S: Storage, A: Api, Q: Querier>(
        deps: &Extern<S, A, Q>,
        market: &HumanAddr
    ) -> StdResult<u64> {
        let market = market.canonize(&deps.api)?;

        let result: Option<u64> = ns_load(
            &deps.storage,
            Self::NS,
            market.as_slice()
        )?;

        match result {
            Some(id) => Ok(id),
            None => Err(StdError::generic_err("Market is not listed."))
        }
    }

    pub fn update<S: Storage, A: Api, Q: Querier, F>(
        deps: &mut Extern<S, A, Q>,
        market: &HumanAddr,
        update: F
    ) -> StdResult<()>
        where F: FnOnce(Market<CanonicalAddr>) -> StdResult<Market<CanonicalAddr>>
    {
        let id = Self::get_id(deps, market)?;

        IterableStorage::new(Self::NS)
            .update_at(&mut deps.storage, id, update)?;
    
        Ok(())
    }

    pub fn list<S: Storage, A: Api, Q: Querier>(
        deps: &Extern<S, A, Q>,
        pagination: Pagination
    ) -> StdResult<Vec<Market<HumanAddr>>> {
        let limit = pagination.limit.min(PAGINATION_LIMIT);

        let iterator = IterableStorage::new(Self::NS)
            .iter(&deps.storage)?
            .skip(pagination.start as usize)
            .take(limit as usize);

        let mut result = Vec::with_capacity(iterator.len());

        for elem in iterator {
            let elem: Market<CanonicalAddr> = elem?;
            result.push(elem.humanize(&deps.api)?);
        }

        Ok(result)
    }

    #[inline]
    fn load(
        storage: &impl Storage,
        index: u64
    ) -> StdResult<Option<Market<CanonicalAddr>>> {
        IterableStorage::new(Self::NS)
            .get_at(storage, index)
    }
}

impl Borrower {
    const NS: &'static [u8] = b"borrowers";

    pub fn new<S: Storage, A: Api, Q: Querier>(
        deps: &Extern<S, A, Q>,
        address: &HumanAddr
    ) -> StdResult<Self> {
        Ok(Self {
            id: BorrowerId::new(deps, address)?
        })
    }

    pub fn from_base64(bin: Binary) -> StdResult<Self> {
        Ok(Self {
            id: BorrowerId::try_from(bin.0)?
        })
    }

    pub fn id(self) -> Binary {
        self.id.into()
    }

    pub fn add_market<S: Storage>(
        &self,
        storage: &mut S,
        id: u64
    ) -> StdResult<()> {
        let mut storage: Bucket<'_, S, u64> =
            Bucket::new(&self.create_key(), storage);

        storage.save(&id.to_be_bytes(), &id)
    }

    pub fn list_markets<S: Storage, A: Api, Q: Querier>(
        &self,
        deps: &Extern<S, A, Q>
    ) -> StdResult<Vec<Market<HumanAddr>>> {
        let storage: ReadonlyBucket<'_, S, u64> =
            ReadonlyBucket::new(&self.create_key(), &deps.storage);

        // Bucket iterator doesn't implement len() :(
        let mut result = Vec::new();

        for item in storage.range(None, None, Order::Ascending) {
            let (_, value) = item?;
            let market = Markets::get_by_id(deps, value)?.unwrap();

            result.push(market);
        }

        Ok(result)
    }

    fn create_key(&self) -> Vec<u8> {
        [ Self::NS, &self.id.0 ].concat()
    }
}

impl BorrowerId {
    const KEY: &'static [u8] = b"salt";

    pub fn new<S: Storage, A: Api, Q: Querier>(
        deps: &Extern<S, A, Q>,
        address: &HumanAddr
    ) -> StdResult<Self> {
        let address = address.canonize(&deps.api)?;
        let salt = Self::load_prng_seed(&deps.storage)?;

        let data = vec![ address.as_slice(), salt.as_slice() ].concat();

        Ok(Self(sha_256(&data)))
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.0
    }

    pub fn set_prng_seed(storage: &mut impl Storage, prng_seed: &Binary) -> StdResult<()> {
        let stored: Option<Binary> = load(storage, Self::KEY)?;

        // Should only set this once, otherwise will break the contract.
        if stored.is_some() {
            return Err(StdError::generic_err("Prng seed already set."));
        }

        save(storage, Self::KEY, prng_seed)
    }

    fn load_prng_seed(storage: &impl Storage) -> StdResult<Binary> {
        Ok(load(storage, Self::KEY)?.unwrap())
    }
}

impl Into<Binary> for BorrowerId {
    fn into(self) -> Binary {
        Binary::from(self.0)
    }
}

impl TryFrom<Vec<u8>> for BorrowerId {
    type Error = StdError;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        match value.try_into() {
            Ok(data) => Ok(Self(data)),
            Err(_) => Err(StdError::generic_err("Couldn't create BorrowerId from bytes."))
        }
    }
}
