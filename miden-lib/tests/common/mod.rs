pub use crypto::{
    hash::rpo::{Rpo256 as Hasher, RpoDigest as Digest},
    merkle::{MerkleStore, NodeIndex, SimpleSmt},
    FieldElement, StarkField, ONE, ZERO,
};
pub use miden_lib::{memory, MidenLib, SatKernel};
pub use miden_objects::{
    assets::{Asset, FungibleAsset, NonFungibleAsset, NonFungibleAssetDetails},
    mock as data,
    mock::assembler,
    notes::{Note, NoteInclusionProof, NoteScript, NoteVault, NOTE_LEAF_DEPTH, NOTE_TREE_DEPTH},
    transaction::{ExecutedTransaction, PreparedTransaction, ProvenTransaction},
    Account, AccountCode, AccountId, AccountStorage, AccountType, AccountVault, BlockHeader,
    ChainMmr, StorageItem,
};
pub use processor::{
    math::Felt, AdviceProvider, ExecutionError, ExecutionOptions, MemAdviceProvider, Process,
    Program, StackInputs, Word,
};
use std::{env, fs::File, io::Read, path::Path};

pub mod procedures;

pub const TX_KERNEL_DIR: &str = "sat/internal";

// TEST BRACE
// ================================================================================================

/// Loads the specified file and append `code` into its end.
pub fn load_file_with_code(imports: &str, code: &str, dir: &str, file: &str) -> String {
    let assembly_file = Path::new(env!("CARGO_MANIFEST_DIR")).join("asm").join(dir).join(file);

    let mut module = String::new();
    File::open(assembly_file).unwrap().read_to_string(&mut module).unwrap();
    let complete_code = format!("{imports}{module}{code}");

    // This hack is going around issue #686 on miden-vm
    complete_code.replace("export", "proc")
}

/// Inject `code` along side the specified file and run it
pub fn run_tx<A>(
    program: Program,
    stack_inputs: StackInputs,
    mut adv: A,
) -> Result<Process<A>, ExecutionError>
where
    A: AdviceProvider,
{
    // mock account method for testing from root context
    adv.insert_into_map(Word::default(), vec![Felt::new(255)]).unwrap();

    let mut process =
        Process::new(program.kernel().clone(), stack_inputs, adv, ExecutionOptions::default());
    process.execute(&program)?;
    Ok(process)
}

/// Inject `code` along side the specified file and run it
pub fn run_within_tx_kernel<A>(
    imports: &str,
    code: &str,
    stack_inputs: StackInputs,
    mut adv: A,
    dir: Option<&str>,
    file: Option<&str>,
) -> Result<Process<A>, ExecutionError>
where
    A: AdviceProvider,
{
    // mock account method for testing from root context
    adv.insert_into_map(Word::default(), vec![Felt::new(255)]).unwrap();

    let assembler = assembler();

    let code = match (dir, file) {
        (Some(dir), Some(file)) => load_file_with_code(imports, code, dir, file),
        (None, None) => format!("{imports}{code}"),
        _ => panic!("both dir and file must be specified"),
    };

    let program = assembler.compile(code).unwrap();

    let mut process =
        Process::new(program.kernel().clone(), stack_inputs, adv, ExecutionOptions::default());
    process.execute(&program)?;
    Ok(process)
}

// TEST HELPERS
// ================================================================================================
pub fn consumed_note_data_ptr(note_idx: u32) -> memory::MemoryAddress {
    memory::CONSUMED_NOTE_SECTION_OFFSET + (1 + note_idx) * 1024
}

pub fn prepare_transaction(
    account: Account,
    account_seed: Option<Word>,
    block_header: BlockHeader,
    chain: ChainMmr,
    notes: Vec<Note>,
    code: &str,
    imports: &str,
    dir: Option<&str>,
    file: Option<&str>,
) -> PreparedTransaction {
    let assembler = assembler();

    let code = match (dir, file) {
        (Some(dir), Some(file)) => load_file_with_code(imports, code, dir, file),
        (None, None) => format!("{imports}{code}"),
        _ => panic!("both dir and file must be specified"),
    };

    let program = assembler.compile(code).unwrap();

    PreparedTransaction::new(account, account_seed, block_header, chain, notes, None, program)
        .unwrap()
}
