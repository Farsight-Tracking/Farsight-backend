use crate::{
    get_provider_manager,
    i_main_registrar::IMainRegistrar,
    types::{
        api::{
            error,
            registration::{self, ChainState},
        },
        contract::ContractType,
    },
};
use actix_web::web::Json;
use ethers::types::U256;
use futures::join;

pub async fn handle(
    req: Json<registration::Request>,
) -> Result<Json<registration::Response>, error::Error> {
    if is_available(req.name).await? {
        return Ok(Json(registration::Response {
            available: true,
            chain_states: vec![],
        }));
    }

    let chain_states = chain_data(req.name).await?;
    return Ok(Json(registration::Response {
        available: false,
        chain_states,
    }));
}

//#[cached(
//    type = "TimedSizedCache<U256, Result<bool, error::Error>>",
//    create = "{ TimedSizedCache::with_size_and_lifespan(DEFAULT_CACHE_SIZE,DEFAULT_CACHE_TIMEOUT) }"
//)]
async fn is_available(name: U256) -> Result<bool, error::Error> {
    let main_provider = get_provider_manager().main();

    let registrar = main_provider
        .contract_address(ContractType::Registrar)
        .unwrap();

    let main_reg = IMainRegistrar::new(registrar.address().clone(), main_provider.provider());
    Ok(main_reg.available(name).call().await?)
}

//#[cached(
//    type = "TimedSizedCache<U256, Result<Vec<ChainState>, error::Error>>",
//    create = "{ TimedSizedCache::with_size_and_lifespan(DEFAULT_CACHE_SIZE,DEFAULT_CACHE_TIMEOUT) }"
//)]
async fn chain_data(name: U256) -> Result<Vec<ChainState>, error::Error> {
    let mut chain_states = vec![];

    let pm = get_provider_manager();

    for provider in pm.provider_iter() {
        let reg_address = provider.contract_address(ContractType::Registrar).unwrap();
        let registrar = crate::IRegistrar::new(reg_address.address().clone(), provider.provider());

        let owner = registrar.get_local_owner_of(name);
        let expires = registrar.get_name_expiration(name);
        let is_keeper = registrar.is_keeper(name);
        let owner_change_version = registrar.get_owner_change_version(name);
        let registration_version = registrar.get_registration_version(name);

        let (owner, expiration, is_keeper, ocv, rv) = join!(
            owner.call(),
            expires.call(),
            is_keeper.call(),
            owner_change_version.call(),
            registration_version.call()
        );

        let expiration = expiration?;
        let is_keeper = is_keeper?;
        let ocv = ocv?;
        let rv = rv?;

        if owner.is_err() {
            let mut state = ChainState::default();
            state.chain_id = U256::from(provider.id());
            chain_states.push(state);
            continue;
        }

        let owner = owner.unwrap();

        let state = ChainState {
            chain_id: U256::from(provider.id()),
            owner,
            expiration,
            is_keeper,
            owner_change_version: ocv,
            registration_version: rv,
        };
        chain_states.push(state);
    }

    Ok(chain_states)
}
