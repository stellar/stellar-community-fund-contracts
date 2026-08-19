use soroban_sdk::{testutils::ContractEvents, xdr, Address, Env, IntoVal, TryFromVal, Val, Vec};

pub fn contract_event<T, D>(
    env: &Env,
    contract_id: &Address,
    topics: T,
    data: D,
) -> xdr::ContractEvent
where
    T: IntoVal<Env, Vec<Val>>,
    D: IntoVal<Env, Val>,
{
    let topics = topics.into_val(env);
    let data = data.into_val(env);
    let contract_id = match xdr::ScAddress::from(contract_id) {
        xdr::ScAddress::Contract(contract_id) => contract_id,
        _ => panic!("expected contract address"),
    };

    xdr::ContractEvent {
        ext: xdr::ExtensionPoint::V0,
        type_: xdr::ContractEventType::Contract,
        contract_id: Some(contract_id),
        body: xdr::ContractEventBody::V0(xdr::ContractEventV0 {
            topics: topics.into(),
            data: xdr::ScVal::try_from_val(env, &data).unwrap(),
        }),
    }
}

pub fn last_events(events: &ContractEvents, count: usize) -> &[xdr::ContractEvent] {
    &events.events()[(events.events().len() - count)..]
}
