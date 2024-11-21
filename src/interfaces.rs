// use rand::prelude::Rng;
use thiserror::Error;
use std::result::Result as StdResult;
use fuels::{
    prelude::*,
    prelude::{
        Address, ContractId,
    },
    types::{
        Bits256,
        transaction::TxPolicies,
        input::Input as SdkInput,
        output::Output as SdkOutput,
    },
    accounts::wallet::WalletUnlocked,
};




#[derive(Error, Debug)]
pub enum GenIOError {
    #[error("No non-zero amount to swap in either input or output")]
    NoAmountToSwap,
}

/// interfaces to SwapVerifier verifier Contract
pub mod generalized_swap_verifier_interface {
    use super::*;

    pub const VERIFIER_CONTRACT_BINARY_PATH: &str = "./contracts/contract_validator/out/debug/contract_validator.bin";
    pub const VERIFIER_CONTRACT_STORAGE_PATH: &str = "./contracts/contract_validator/out/debug/contract_validator-storage_slots.json";

    abigen!(
        Contract(
        name = "SwapVerifier",
        abi = "./contracts/contract_validator/out/debug/contract_validator-abi.json"
        ),
    );


    pub fn get_contract_verifier_path() -> (String, String) {
        (VERIFIER_CONTRACT_BINARY_PATH.to_string(), VERIFIER_CONTRACT_STORAGE_PATH.to_string())
    }


    pub async fn contract_verifier_instance(
        wallet: WalletUnlocked,
    ) -> (SwapVerifier<WalletUnlocked>, ContractId) {

        // deploy with salt:
        // let mut rng = rand::thread_rng();
        // let salt = rng.gen::<[u8; 32]>();
        let salt: [u8; 32] = [01u8; 32];
        println!("SwapVerifier salt: {}", hex::encode(salt));

        let storage_configuration = StorageConfiguration::default()
            .add_slot_overrides_from_file(VERIFIER_CONTRACT_STORAGE_PATH)
            .unwrap();

        let configuration = LoadConfiguration::default()
            .with_storage_configuration(storage_configuration)
            .with_salt(salt);

        let contract_id_bech32 = Contract::load_from(
            VERIFIER_CONTRACT_BINARY_PATH,
            configuration,
            )
            .unwrap()
            .deploy(&wallet, TxPolicies::default())
            .await
            .unwrap();

        let cv_instance = SwapVerifier::new(contract_id_bech32.clone(), wallet);

        let cv_contract_id = ContractId::from_bytes_ref(&contract_id_bech32.hash);

        println!("SwapVerifier ID bech32 \t: {}", contract_id_bech32.clone().to_string());
        println!("SwapVerifier ID (hex) \t: {}", cv_contract_id);


        (cv_instance, contract_id_bech32.into())
    }

    /// helper function to simply populate the GenIO tx data struct with the tx params.
    pub fn populate_genio(
        assets_in: [Bits256; 5],
        utxoids_in: [Bits256; 5],
        amounts_in: [Bits256; 5],
        asset_out: Bits256,
        amount_out: Bits256,
        tolerance_bps: Bits256,
    ) -> StdResult<GenIO, GenIOError> {
        if !amounts_in.iter().any(|&amount| amount != Bits256::zeroed()) {
            return Err(GenIOError::NoAmountToSwap);
        }
        let generalio_tx = GenIO {
            inputassets: assets_in,
            inpututxoids: utxoids_in,
            inputamounts: amounts_in,
            outputasset: asset_out,
            outputamount: amount_out,
            tolerance: tolerance_bps,
        };
        Ok(generalio_tx)
    }

    pub async fn build_verify_intent_sender_tx(
        contract_instance: SwapVerifier<WalletUnlocked>,
        gio_data: GenIO,
        signature_bytes: Vec<u8>,
        tx_policy: TxPolicies,
        gas_input: SdkInput,
        gas_change: SdkOutput,
    ) -> ScriptTransactionBuilder {

        let mut stb = contract_instance
            .methods()
            .verify_intent_sender(
                gio_data,
                Bytes(signature_bytes),
            )
            .with_tx_policies(tx_policy)
            .transaction_builder().await.unwrap();

        // add the gas input and output
        stb.inputs_mut().push(gas_input);
        stb.outputs_mut().push(gas_change);

        stb
    }

    pub async fn call_validate_solution(
        contract_instance: SwapVerifier<WalletUnlocked>,
        tx_sender: Bits256,
        gio_data: GenIO,
        signature_bytes: Vec<u8>,
        inputs: Vec<SdkInput>,
        outputs: Vec<SdkOutput>,
    ) -> ScriptTransactionBuilder {

        // .with_script_gas_limit(840000)

        let intent = Intent {
            sender: tx_sender,
            compsig: Bytes(signature_bytes),
            io: gio_data,
        };
        let mut stb = contract_instance
            .methods()
            .validate_solution(
                intent,
            )
            .with_tx_policies(TxPolicies::default())
            .transaction_builder().await.unwrap();

        for some_input in inputs {
            stb.inputs_mut().push(some_input);
        }
        for some_output in outputs {
            stb.outputs_mut().push(some_output);
        }

        stb
    }



}






/*
pub mod predicate_validator {
    use super::*;

    pub const PREDICATE_VALIDATOR_BINARY_PATH: &str = "./contracts/predicate_validator/out/debug/predicate_validator.bin";

    abigen!(
        Predicate(
            name = "SimpleSwapPredicateValidator",
            abi = "./contracts/predicate_validator/out/debug/predicate_validator-abi.json"
        ),
    );

    pub fn get_predicate_validator_data(
        sender: Bits256,
        signature_bytes: Vec<u8>,
        asset_in: Bits256,
        amount_in: Bits256,
        utxoid_in: Bits256,
        asset_out: Bits256,
        amount_out: Bits256,
    ) -> Vec<u8> {

        let mut encoded_bytes: Vec<u8> = Vec::new();
        encoded_bytes.push(0x00);
        let predicate_data = SimpleSwapPredicateValidatorEncoder::default().encode_data(
            Bytes(encoded_bytes),
        ).unwrap();
        predicate_data
    }


    pub fn get_predicate_validator_info(
        // configurable_byte: u8,
    ) -> (Vec<u8>, Bech32Address, Address) {

        let predicate_validator_bytecode = std::fs::read(PREDICATE_VALIDATOR_BINARY_PATH).unwrap();
        let predicate = Predicate::from_code(predicate_validator_bytecode.clone());

        // let configurables = get_predicate_validator_configurables(
        //     configurable_byte,
        // );
        // let predicate = Predicate::from_code(predicate_validator_bytecode.clone())
        //     .with_configurables(configurables);

        let predicate_b32addr = predicate.address().clone();
        let predicate_address: Address = predicate.address().into();
        let new_bytecode = predicate.code().to_vec();

        (new_bytecode, predicate_b32addr, predicate_address)
    }

}
*/

// some basic utils for a dummy predicate wallet that always returns true.
pub mod dummy_pwallet {
    // use fuels::types::Bits256;
    // use fuels::types::EvmAddress;

    use super::*;

    pub const PREDICATE_DUMMYPWALLET_BINARY_PATH: &str = "./contracts/dummy_predicate_wallet/out/debug/dummy_predicate_wallet.bin";

    abigen!(
        Predicate(
            name = "DummyPredicateWallet",
            abi = "./contracts/dummy_predicate_wallet/out/debug/dummy_predicate_wallet-abi.json"
        ),
    );

    pub fn get_dummypwallet_configurables(
        configurable_byte: u8,
    ) -> DummyPredicateWalletConfigurables {

        let configurables = DummyPredicateWalletConfigurables::default()
            .with_RANDOMBYTE(configurable_byte).unwrap();
        configurables
    }

    pub fn get_dummypwallet_info(
        configurable_byte: u8,
    ) -> (Vec<u8>, Bech32Address, Address) {

        let dummypwallet_bytecode = std::fs::read(PREDICATE_DUMMYPWALLET_BINARY_PATH).unwrap();
        let configurables = get_dummypwallet_configurables(
            configurable_byte,
        );
        let predicate = Predicate::from_code(dummypwallet_bytecode.clone())
            .with_configurables(configurables);

        let predicate_b32addr = predicate.address().clone();
        let predicate_address: Address = predicate.address().into();
        let predicate_bytecode = predicate.code().to_vec();

        (predicate_bytecode, predicate_b32addr, predicate_address)
    }



}


pub mod tokenminter {
    use super::*;
    use fuels::types::Bits256;

    pub const CONTRACT_TOKENMINTER_BINARY_PATH: &str = "./contracts/asset_ops/out/debug/asset_ops.bin";
    pub const CONTRACT_TOKENMINTER_STORAGEJSON_PATH: &str = "./contracts/asset_ops/out/debug/asset_ops-storage_slots.json";


    abigen!(
        Contract(
            name = "TokenMinter",
            abi = "./contracts/asset_ops/out/debug/asset_ops-abi.json"
        ),
    );

    pub async fn deploy_tokenminter(
        wallet_with_gas: &WalletUnlocked,
    ) -> ContractId {

        // deploy with salt:
        // let mut rng = rand::thread_rng();
        // let salt = rng.gen::<[u8; 32]>();
        let salt: [u8; 32] = [02u8; 32];
        println!("TokenMinter salt: {}", hex::encode(salt));

        let storage_configuration = StorageConfiguration::default()
            .add_slot_overrides_from_file(CONTRACT_TOKENMINTER_STORAGEJSON_PATH)
            .unwrap();

        let configuration = LoadConfiguration::default()
            .with_storage_configuration(storage_configuration)
            .with_salt(salt);

        let tokenminter_b32cid = Contract::load_from(
            CONTRACT_TOKENMINTER_BINARY_PATH,
            configuration,
        )
        .unwrap()
        .deploy(wallet_with_gas, TxPolicies::default())
        .await
        .unwrap();

        let tokenminter_cid = ContractId::from_bytes_ref(&tokenminter_b32cid.hash);
        *tokenminter_cid
    }

    pub async fn mint_to(
        tokenminter_cid: ContractId,
        wallet_with_gas: &WalletUnlocked,
        receiver: Address,
        amount: u64,
        sub_id: Bits256,
    ) {
        let tokenminter_instance = TokenMinter::new(
            tokenminter_cid.clone(),
            wallet_with_gas.clone()
        );
        let _fsh = tokenminter_instance
            .methods()
            .mint_and_send_to_address(amount, receiver, sub_id)
            .with_variable_output_policy(VariableOutputPolicy::Exactly(1))
            .call()
            .await
            .unwrap();
    }

}

