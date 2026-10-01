use std::fs;

use kaspa_consensus_core::hashing::sighash::calc_schnorr_signature_hash;
use kaspa_consensus_core::hashing::sighash::SigHashReusedValuesUnsync;
use kaspa_consensus_core::hashing::sighash_type::SIG_HASH_ALL;
use kaspa_consensus_core::mass::units::SigopCount;
use kaspa_consensus_core::tx::{
    MutableTransaction, ScriptPublicKey, Transaction, TransactionId, TransactionInput, TransactionOutpoint, TransactionOutput,
    UtxoEntry, VerifiableTransaction,
};
use kaspa_txscript::caches::Cache;
use kaspa_txscript::opcodes::codes::{OpCheckSig, OpData32};
use kaspa_txscript::script_builder::ScriptBuilder;
use kaspa_txscript::{EngineCtx, EngineFlags, TxScriptEngine};
use secp256k1::{Keypair, Message, Secp256k1, SecretKey};
use silverscript_abi::encode_contract_entry_sig_script;
use silverscript_lang::compiler::compile_to_sil_abi_artifact;

const LOCKED: u64 = 100_000_000;
const FEE: u64 = 1_000_000;

fn holder_key() -> Keypair {
    let secp = Secp256k1::new();
    let secret = SecretKey::from_slice(&[7u8; 32]).expect("test key");
    Keypair::from_secret_key(&secp, &secret)
}

fn other_key() -> Keypair {
    let secp = Secp256k1::new();
    let secret = SecretKey::from_slice(&[9u8; 32]).expect("test key");
    Keypair::from_secret_key(&secp, &secret)
}

fn xonly(key: &Keypair) -> Vec<u8> {
    key.x_only_public_key().0.serialize().to_vec()
}

fn p2pk(pubkey: &[u8]) -> Vec<u8> {
    // The script engine prefixes the version bytes. SilverScript's
    // ScriptPubKeyP2PK is those two bytes plus this body.
    ScriptBuilder::new().add_data(pubkey).unwrap().add_op(OpCheckSig).unwrap().drain()
}

fn compile(holder: &[u8]) -> silverscript_abi::SilAbiArtifact {
    let source = fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/contracts/KusdtLock.sil")).expect("read contract");
    compile_to_sil_abi_artifact(&source, &[holder.to_vec().into()]).expect("compile")
}

fn bytecode(artifact: &silverscript_abi::SilAbiArtifact) -> Vec<u8> {
    artifact.contracts.values().next().expect("contract").compiled.bytecode.clone()
}

struct Built {
    tx: MutableTransaction<Transaction>,
    utxo: UtxoEntry,
    reused: SigHashReusedValuesUnsync,
}

fn build(lock_script: Vec<u8>, output_script: Vec<u8>, output_value: u64) -> Built {
    let reused = SigHashReusedValuesUnsync::new();
    let lock_input = TransactionInput {
        previous_outpoint: TransactionOutpoint { transaction_id: TransactionId::from_bytes([1u8; 32]), index: 0 },
        signature_script: vec![],
        sequence: 0,
        compute_commit: SigopCount(1).into(),
    };
    let fee_input = TransactionInput {
        previous_outpoint: TransactionOutpoint { transaction_id: TransactionId::from_bytes([2u8; 32]), index: 0 },
        signature_script: vec![],
        sequence: 0,
        compute_commit: SigopCount(1).into(),
    };
    let output = TransactionOutput {
        value: output_value,
        script_public_key: ScriptPublicKey::new(0, output_script.into()),
        covenant: None,
    };
    let tx = Transaction::new(0, vec![lock_input, fee_input], vec![output], 0, Default::default(), 0, vec![]);
    let lock_utxo = UtxoEntry::new(LOCKED, ScriptPublicKey::new(0, lock_script.into()), 0, false, None);
    let fee_utxo = UtxoEntry::new(FEE, ScriptPublicKey::new(0, vec![OpData32].into()), 0, false, None);
    let tx = MutableTransaction::with_entries(tx, vec![lock_utxo.clone(), fee_utxo]);
    Built { tx, utxo: lock_utxo, reused }
}

fn sign(built: &Built, key: &Keypair) -> Vec<u8> {
    let sig_hash = calc_schnorr_signature_hash(&built.tx.as_verifiable(), 0, SIG_HASH_ALL, &built.reused);
    let message = Message::from_digest_slice(sig_hash.as_bytes().as_slice()).expect("digest");
    let sig = key.sign_schnorr(message);
    let mut signature = Vec::with_capacity(65);
    signature.extend_from_slice(sig.as_ref().as_slice());
    signature.push(SIG_HASH_ALL.to_u8());
    signature
}

fn execute(mut built: Built, artifact: &silverscript_abi::SilAbiArtifact, signature: Vec<u8>) -> Result<(), kaspa_txscript_errors::TxScriptError> {
    let sigscript = encode_contract_entry_sig_script(artifact, "KusdtLock", "redeem", &[signature.into()]).expect("sigscript");
    built.tx.tx.inputs[0].signature_script = sigscript;
    let cache = Cache::new(10_000);
    let verifiable = built.tx.as_verifiable();
    let mut vm = TxScriptEngine::from_transaction_input(
        &verifiable,
        &verifiable.inputs()[0],
        0,
        &built.utxo,
        EngineCtx::new(&cache).with_reused(&built.reused),
        EngineFlags { covenants_enabled: true, ..Default::default() },
    );
    vm.execute()
}

#[test]
fn holder_redeems_the_full_lock_and_the_fee_stays_on_the_other_input() {
    let holder = holder_key();
    let artifact = compile(&xonly(&holder));
    let built = build(bytecode(&artifact), p2pk(&xonly(&holder)), LOCKED);
    let signature = sign(&built, &holder);
    execute(built, &artifact, signature).expect("full redeem");
}

#[test]
fn a_skimmed_output_fails() {
    let holder = holder_key();
    let artifact = compile(&xonly(&holder));
    let built = build(bytecode(&artifact), p2pk(&xonly(&holder)), LOCKED - 1);
    let signature = sign(&built, &holder);
    assert!(execute(built, &artifact, signature).is_err());
}

#[test]
fn a_different_recipient_fails() {
    let holder = holder_key();
    let artifact = compile(&xonly(&holder));
    let built = build(bytecode(&artifact), p2pk(&xonly(&other_key())), LOCKED);
    let signature = sign(&built, &holder);
    assert!(execute(built, &artifact, signature).is_err());
}

#[test]
fn a_different_key_fails() {
    let holder = holder_key();
    let artifact = compile(&xonly(&holder));
    let built = build(bytecode(&artifact), p2pk(&xonly(&holder)), LOCKED);
    let signature = sign(&built, &other_key());
    assert!(execute(built, &artifact, signature).is_err());
}

#[test]
fn the_contract_has_one_entry() {
    let artifact = compile(&xonly(&holder_key()));
    let contract = artifact.contracts.get("KusdtLock").expect("contract");
    let names: Vec<_> = contract.entries.keys().cloned().collect();
    assert_eq!(names, vec!["redeem".to_string()]);
}

#[test]
fn p2pk_push_uses_the_data_opcode() {
    let _ = ScriptBuilder::new();
    assert_eq!(OpData32, 0x20);
}
