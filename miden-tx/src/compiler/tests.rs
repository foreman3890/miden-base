use super::{AccountId, ModuleAst, Note, NoteTarget, Operation, ProgramAst, TransactionComplier};
use assembly::AssemblyContext;
use crypto::{Felt, FieldElement, Word};
use miden_core::code_blocks::CodeBlock;
use miden_objects::assets::{Asset, FungibleAsset};

// CONSTANTS
// ================================================================================================

const ACCOUNT_ID_REGULAR_ACCOUNT_IMMUTABLE_CODE_ON_CHAIN: u64 = 0b0110011011u64 << 54;

// Mast roots of account procedures:
const ACCT_PROC_1: &'static str =
    "0x8ef0092134469a1330e3c468f57c7f085ce611645d09cc7516c786fefc71d794";
const ACCT_PROC_2: &'static str =
    "0xff06b90f849c4b262cbfbea67042c4ea017ea0e9c558848a951d44b23370bec5";
const ACCOUNT_CODE_MASM: &'static str = "\
export.account_procedure_1
    push.1.2
    add
end

export.account_procedure_2
    push.2.1
    sub
end
";

// Mast roots of additional procedures:
const ADD_PROC_1: &'static str =
    "0x5b6f7afcde4aaf538519c3bf5bb9321fac83cd769a3100c0b1225c9a6d75c9a1";
const ADD_PROC_2: &'static str =
    "0xd4b1f9fbad5d0e6d2386509eab6a865298db20095d7315226dfa513ce017c990";
const ADDITIONAL_PROCEDURES: &'static str = "\
export.additional_procedure_1
    push.3.4
    add
end

export.additional_procedure_2
    push.4.5
    add
end
";

// TESTS
// ================================================================================================

#[test]
fn test_load_account() {
    let mut tx_compiler = TransactionComplier::new();
    let account_id =
        AccountId::try_from(ACCOUNT_ID_REGULAR_ACCOUNT_IMMUTABLE_CODE_ON_CHAIN).unwrap();
    let account_code_ast = ModuleAst::parse(ACCOUNT_CODE_MASM).unwrap();
    let account_code = tx_compiler.load_account(account_id, account_code_ast).unwrap();

    let acct_procs = [hex_to_bytes(ACCT_PROC_1), hex_to_bytes(ACCT_PROC_2)];
    for proc in account_code.procedures() {
        assert!(acct_procs.contains(&proc.as_bytes().to_vec()));
    }
}

#[test]
fn test_compile_valid_note_script() {
    let test_cases = [
        (
            format!(
                "begin
                    call.{ACCT_PROC_1}
                    call.{ACCT_PROC_2}
                end"
            ),
            true,
        ),
        (
            format!(
                "begin
                    if.true
                        call.{ACCT_PROC_1}
                        if.true
                            call.{ACCT_PROC_2}
                        else
                            call.{ADD_PROC_1}
                        end
                    else
                        call.{ADD_PROC_2}
                    end
                end"
            ),
            true,
        ),
        (
            format!(
                "begin
                    call.{ACCT_PROC_1}
                    if.true
                        call.{ADD_PROC_1}
                    else
                        call.{ADD_PROC_2}
                    end
                end"
            ),
            false,
        ),
    ];

    let mut tx_compiler = TransactionComplier::new();
    let account_id =
        AccountId::try_from(ACCOUNT_ID_REGULAR_ACCOUNT_IMMUTABLE_CODE_ON_CHAIN).unwrap();
    let account_code_ast = ModuleAst::parse(ACCOUNT_CODE_MASM).unwrap();
    let _account_code = tx_compiler.load_account(account_id, account_code_ast).unwrap();
    let target_account_proc = NoteTarget::AccountId(account_id);

    // TODO: replace this with anonymous call targets once they are implemented
    let account_id =
        AccountId::try_from(ACCOUNT_ID_REGULAR_ACCOUNT_IMMUTABLE_CODE_ON_CHAIN + 1).unwrap();
    let account_code_ast = ModuleAst::parse(ADDITIONAL_PROCEDURES).unwrap();
    tx_compiler.load_account(account_id, account_code_ast).unwrap();

    for (note_script_src, expected) in test_cases {
        let note_script_ast = ProgramAst::parse(note_script_src.as_str()).unwrap();

        let result =
            tx_compiler.compile_note_script(note_script_ast, vec![target_account_proc.clone()]);
        match expected {
            true => assert!(result.is_ok()),
            false => assert!(result.is_err()),
        }
    }
}

fn mock_consumed_notes(
    tx_compiler: &mut TransactionComplier,
    target_account: AccountId,
) -> Vec<Note> {
    pub const ACCOUNT_ID_SENDER: u64 = 0b0110111011u64 << 54;

    pub const ACCOUNT_ID_FUNGIBLE_FAUCET_ON_CHAIN: u64 = 0b1010011100 << 54;
    // Note Assets
    let faucet_id_1 = AccountId::try_from(ACCOUNT_ID_FUNGIBLE_FAUCET_ON_CHAIN).unwrap();
    let faucet_id_2 = AccountId::try_from(ACCOUNT_ID_FUNGIBLE_FAUCET_ON_CHAIN + 10).unwrap();
    let faucet_id_3 = AccountId::try_from(ACCOUNT_ID_FUNGIBLE_FAUCET_ON_CHAIN + 20).unwrap();
    let fungible_asset_1: Asset = FungibleAsset::new(faucet_id_1, 100).unwrap().into();
    let fungible_asset_2: Asset = FungibleAsset::new(faucet_id_2, 200).unwrap().into();
    let fungible_asset_3: Asset = FungibleAsset::new(faucet_id_3, 300).unwrap().into();

    // Sender account
    let sender = AccountId::try_from(ACCOUNT_ID_SENDER).unwrap();

    // create note script
    let note_program_ast =
        ProgramAst::parse(format!("begin call.{ACCT_PROC_1} drop end").as_str()).unwrap();
    let note_script = tx_compiler
        .compile_note_script(note_program_ast, vec![NoteTarget::AccountId(target_account)])
        .unwrap();

    // Consumed Notes
    const SERIAL_NUM_1: Word = [Felt::new(1), Felt::new(2), Felt::new(3), Felt::new(4)];
    let note_1 = Note::new(
        note_script.clone(),
        &[Felt::new(1)],
        &[fungible_asset_1, fungible_asset_2, fungible_asset_3],
        SERIAL_NUM_1,
        sender,
        Felt::ZERO,
        None,
    )
    .unwrap();

    const SERIAL_NUM_2: Word = [Felt::new(5), Felt::new(6), Felt::new(7), Felt::new(8)];
    let note_2 = Note::new(
        note_script,
        &[Felt::new(2)],
        &[fungible_asset_1, fungible_asset_2, fungible_asset_3],
        SERIAL_NUM_2,
        sender,
        Felt::ZERO,
        None,
    )
    .unwrap();

    vec![note_1, note_2]
}

#[test]
fn test_transaction_compilation() {
    let mut tx_compiler = TransactionComplier::new();
    let account_id =
        AccountId::try_from(ACCOUNT_ID_REGULAR_ACCOUNT_IMMUTABLE_CODE_ON_CHAIN).unwrap();
    let account_code_ast = ModuleAst::parse(ACCOUNT_CODE_MASM).unwrap();
    let _account_code = tx_compiler.load_account(account_id, account_code_ast).unwrap();

    let notes = mock_consumed_notes(&mut tx_compiler, account_id);

    let tx_script_src = format!("begin call.{ACCT_PROC_2} end");
    let tx_script_ast = ProgramAst::parse(tx_script_src.as_str()).unwrap();

    let (tx_program, _) = tx_compiler
        .compile_transaction(account_id, &notes, Some(tx_script_ast.clone()))
        .unwrap();

    assert_eq!(
        tx_program.root().hash(),
        expected_mast_tree(&mut tx_compiler, &notes, &tx_script_ast).hash()
    );
}

// HELPERS
// ================================================================================================

fn expected_mast_tree(
    tx_compiler: &mut TransactionComplier,
    notes: &[Note],
    tx_script: &ProgramAst,
) -> CodeBlock {
    let noop_span = CodeBlock::new_span(vec![Operation::Noop]);

    let note_0_leafs = create_note_leafs(tx_compiler, &notes[0]);
    let note_1_leafs = create_note_leafs(tx_compiler, &notes[1]);
    let note_teardown_leafs =
        CodeBlock::new_join([tx_compiler.note_processing_teardown.clone(), noop_span.clone()]);

    let tx_script_program = tx_compiler
        .assembler
        .compile_in_context(
            tx_script,
            &mut AssemblyContext::new(assembly::AssemblyContextType::Program),
        )
        .unwrap();
    let tx_script_epilogue_leafs = CodeBlock::new_join([
        CodeBlock::new_call(tx_script_program.hash()),
        tx_compiler.epilogue.clone(),
    ]);

    let note_tree_internal_node = CodeBlock::new_join([note_0_leafs, note_1_leafs]);
    let note_tree_root = CodeBlock::new_join([note_tree_internal_node, note_teardown_leafs]);

    let prologue_and_note_tree_node =
        CodeBlock::new_join([tx_compiler.prologue.clone(), note_tree_root]);

    let program_root = CodeBlock::new_join([prologue_and_note_tree_node, tx_script_epilogue_leafs]);

    program_root
}

fn create_note_leafs(tx_compiler: &mut TransactionComplier, note: &Note) -> CodeBlock {
    let note_code_block = tx_compiler
        .assembler
        .compile_in_context(
            note.script().code(),
            &mut AssemblyContext::new(assembly::AssemblyContextType::Program),
        )
        .unwrap();
    CodeBlock::new_join([
        tx_compiler.note_setup.clone(),
        CodeBlock::new_call(note_code_block.hash()),
    ])
}

fn hex_to_bytes(hex: &str) -> Vec<u8> {
    (2..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
        .collect::<Vec<_>>()
}
