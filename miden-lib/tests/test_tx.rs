pub mod common;
use common::{
    data::ACCOUNT_ID_FUNGIBLE_FAUCET_ON_CHAIN,
    memory::{
        CREATED_NOTE_ASSETS_OFFSET, CREATED_NOTE_METADATA_OFFSET, CREATED_NOTE_RECIPIENT_OFFSET,
        CREATED_NOTE_SECTION_OFFSET, NUM_CREATED_NOTES_PTR,
    },
    procedures::prepare_word,
    run_within_tx_kernel, Felt, MemAdviceProvider, StackInputs, ONE, ZERO,
};

#[test]
fn test_create_note() {
    let recipient = [ZERO, ONE, Felt::new(2), Felt::new(3)];
    let tag = Felt::new(4);
    let asset = [Felt::new(10), ZERO, ZERO, Felt::new(ACCOUNT_ID_FUNGIBLE_FAUCET_ON_CHAIN)];

    let code = format!(
        "
    use.miden::sat::tx

    begin
        push.{recipient}
        push.{tag}
        push.{asset}

        exec.tx::create_note
    end
    ",
        recipient = prepare_word(&recipient),
        tag = tag,
        asset = prepare_word(&asset)
    );

    let process = run_within_tx_kernel(
        "",
        &code,
        StackInputs::default(),
        MemAdviceProvider::default(),
        None,
        None,
    )
    .unwrap();

    // assert the number of created notes has been incremented to 1.
    assert_eq!(
        process.get_memory_value(0, NUM_CREATED_NOTES_PTR).unwrap(),
        [ONE, ZERO, ZERO, ZERO]
    );

    // assert the recipient is stored at the correct memory location.
    assert_eq!(
        process
            .get_memory_value(0, CREATED_NOTE_SECTION_OFFSET + CREATED_NOTE_RECIPIENT_OFFSET)
            .unwrap(),
        recipient
    );

    // assert the metadata is stored at the correct memory location.
    assert_eq!(
        process
            .get_memory_value(0, CREATED_NOTE_SECTION_OFFSET + CREATED_NOTE_METADATA_OFFSET)
            .unwrap(),
        [ONE, tag, ZERO, ZERO]
    );

    // assert the asset is stored at the correct memory location.
    assert_eq!(
        process
            .get_memory_value(0, CREATED_NOTE_SECTION_OFFSET + CREATED_NOTE_ASSETS_OFFSET)
            .unwrap(),
        asset
    );

    // assert there top item on the stack is a pointer to the created note.
    assert_eq!(process.stack.get(0), Felt::new(10000));
}

#[test]
fn test_create_note_too_many_notes() {
    let recipient = [ZERO, ONE, Felt::new(2), Felt::new(3)];
    let tag = Felt::new(4);
    let asset = [Felt::new(10), ZERO, ZERO, Felt::new(ACCOUNT_ID_FUNGIBLE_FAUCET_ON_CHAIN)];

    let code = format!(
        "
    use.miden::sat::internal::constants
    use.miden::sat::internal::layout
    use.miden::sat::tx

    begin
        exec.constants::get_max_num_created_notes
        exec.layout::set_num_created_notes

        push.{recipient}
        push.{tag}
        push.{asset}

        exec.tx::create_note
    end
    ",
        recipient = prepare_word(&recipient),
        tag = tag,
        asset = prepare_word(&asset)
    );

    let process = run_within_tx_kernel(
        "",
        &code,
        StackInputs::default(),
        MemAdviceProvider::default(),
        None,
        None,
    );

    // assert the process failed
    assert!(process.is_err());
}
