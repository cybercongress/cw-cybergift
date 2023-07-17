use cosmwasm_std::{Addr, Order, StdError, StdResult, Storage};
use cosmwasm_std::Order::Ascending;
use cosmwasm_std::testing::mock_dependencies;
use cw_storage_plus::{Bound, Index, IndexList, IndexedMap, MultiIndex, PrimaryKey};

use crate::state::{Refer, REFERRALS_PREFIX};

pub const DEFAULT_DEPTH: u64 = 3;
pub const DEFAULT_REFERRED_LIMIT: u64 = 50;
pub const DEFAULT_ALL_LIMIT: u64 = 100;

pub struct ReferralIndexes<'a> {
    pub referred: MultiIndex<'a, Addr, Refer, Addr>,
}

impl IndexList<Refer> for ReferralIndexes<'_> {
    fn get_indexes(&'_ self) -> Box<dyn Iterator<Item = &'_ dyn Index<Refer>> + '_> {
        let v: Vec<&dyn Index<Refer>> = vec![&self.referred];
        Box::new(v.into_iter())
    }
}

#[allow(non_snake_case)]
pub fn REFERRALS<'a>() -> IndexedMap<'a, &'a Addr, Refer, ReferralIndexes<'a>> {
    let indexes = ReferralIndexes {
        referred: MultiIndex::new(
            |refer| refer.referrer.clone(),
            REFERRALS_PREFIX,
            "ref_idx",
        ),
    };
    IndexedMap::new(REFERRALS_PREFIX, indexes)
}

pub fn set_ref(
    storage: &mut dyn Storage,
    referred_addr: &Addr,
    referrer_addr: &Addr,
) -> StdResult<()> {
    if referred_addr == referrer_addr {
        return Err(StdError::generic_err(
            "Referrer can not be same address as referral",
        ));
    }

    REFERRALS().save(
        storage,
        referred_addr,
        &Refer {
            referrer: referrer_addr.clone(),
            referred: referred_addr.clone(),
        },
    )?;

    Ok(())
}

pub fn ref_chains(
    storage: &dyn Storage,
    addr: &Addr,
    depth: Option<u64>,
) -> StdResult<Vec<Addr>> {
    let mut chains: Vec<Addr> = vec![];

    for _ in 0..depth.unwrap_or(DEFAULT_DEPTH) {
        match REFERRALS().may_load(storage, chains.last().unwrap_or(addr))? {
            Some(r_addr) => chains.push(r_addr.referrer),
            None => break,
        };
    }

    Ok(chains)
}

pub fn ref_of(storage: &dyn Storage, addr: &Addr) -> StdResult<Option<Addr>> {
    Ok(REFERRALS().may_load(storage, addr)?.map(|r| r.referrer))
}

pub fn has_ref(storage: &dyn Storage, addr: &Addr) -> StdResult<bool> {
    Ok(ref_of(storage, addr)?.is_some())
}

pub fn all_ref(
    storage: &dyn Storage,
    start_after: Option<Addr>,
    limit: Option<u64>,
    is_ascending: Option<bool>,
) -> Vec<Refer> {
    let bound = match is_ascending.unwrap_or(true) {
        true => (
            start_after
                .as_ref()
                .map(|e| Bound::ExclusiveRaw(e.as_bytes().to_vec())),
            None,
            Order::Ascending,
        ),
        false => (
            None,
            start_after
                .as_ref()
                .map(|e| Bound::ExclusiveRaw(e.as_bytes().to_vec())),
            Order::Descending,
        ),
    };

    REFERRALS()
        .range(storage, bound.0, bound.1, bound.2)
        .map(|e| e.unwrap().1)
        .take(limit.unwrap_or(DEFAULT_ALL_LIMIT) as usize)
        .collect::<Vec<_>>()
}

pub fn all_referred_of(
    storage: &dyn Storage,
    addr: Addr,
    start_after: Option<Addr>,
    limit: Option<u64>,
    is_ascending: Option<bool>,
) -> StdResult<Vec<Addr>> {
    let bound = match is_ascending.unwrap_or(true) {
        true => (
            start_after
                .as_ref()
                .map(|e| Bound::ExclusiveRaw(e.as_bytes().to_vec())),
            None,
            Order::Ascending,
        ),
        false => (
            None,
            start_after
                .as_ref()
                .map(|e| Bound::ExclusiveRaw(e.as_bytes().to_vec())),
            Order::Descending,
        ),
    };

    let res = REFERRALS()
        .idx
        .referred
        .prefix(addr)
        .keys(storage, bound.0, bound.1, bound.2)
        .map(|e| e.unwrap())
        .take(limit.unwrap_or(DEFAULT_REFERRED_LIMIT) as usize)
        .collect::<Vec<_>>();

    Ok(res)
}

#[test]
fn test_indexed_referral() {
    let mut deps = mock_dependencies();

    let a = Addr::unchecked("a");
    let bb = Addr::unchecked("bb");
    let b = Addr::unchecked("b");
    let c = Addr::unchecked("c");
    let d = Addr::unchecked("d");

    set_ref(&mut deps.storage, &b, &a).unwrap();
    set_ref(&mut deps.storage, &bb, &a).unwrap();
    set_ref(&mut deps.storage, &a, &a).unwrap_err();

    let chains = ref_chains(&deps.storage, &b, Some(5)).unwrap();

    assert_eq!(chains, vec![a.clone()]);

    set_ref(&mut deps.storage, &c, &b).unwrap();
    set_ref(&mut deps.storage, &d, &c).unwrap();

    let chains = ref_chains(&deps.storage, &d, Some(5)).unwrap();

    assert_eq!(chains, vec![c.clone(), b.clone(), a.clone()]);

    let e = Addr::unchecked("e");
    let f = Addr::unchecked("f");
    let g = Addr::unchecked("g");

    set_ref(&mut deps.storage, &e, &d).unwrap();
    set_ref(&mut deps.storage, &f, &e).unwrap();
    set_ref(&mut deps.storage, &g, &f).unwrap();

    let chains = ref_chains(&deps.storage, &g, Some(5)).unwrap();

    assert_eq!(chains, vec![f, e, d, c, b.clone()]);
    assert_eq!(chains.len(), 5);

    let h = Addr::unchecked("h");
    let j = Addr::unchecked("j");
    set_ref(&mut deps.storage, &h, &a).unwrap();
    set_ref(&mut deps.storage, &j, &a).unwrap();

    let all_ref_a =
        all_referred_of(&deps.storage, a.clone(), None, None, None)
        .unwrap();

    assert_eq!(all_ref_a, vec![b.clone(), bb.clone(), h.clone(), j.clone()]);

    let all_ref_a =
        all_referred_of(&deps.storage, a.clone(), Some(b.clone()), None, None)
        .unwrap();

    assert_eq!(all_ref_a, vec![bb.clone(), h.clone(), j.clone()]);

    let all_ref_a =
        all_referred_of(&deps.storage, a.clone(), None, None, Some(false))
        .unwrap();

    assert_eq!(all_ref_a, vec![j.clone(), h.clone(), bb.clone(), b.clone()]);

    let all_ref_a =
        all_referred_of(&deps.storage, a.clone(), Some(j.clone()), None, Some(false))
        .unwrap();

    assert_eq!(all_ref_a, vec![h.clone(), bb.clone(), b.clone()]);
}
