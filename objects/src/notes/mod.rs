use super::{
    assets::Asset, AccountId, Assembler, AssemblyContext, AssemblyContextType, CodeBlock, Digest,
    Felt, Hasher, NoteError, ProgramAst, ToString, Vec, Word, WORD_SIZE, ZERO,
};

mod envelope;
pub use envelope::NoteEnvelope;

mod inputs;
use inputs::NoteInputs;

mod metadata;
pub use metadata::NoteMetadata;

mod origin;
pub use origin::{NoteInclusionProof, NoteOrigin};

mod script;
pub use script::NoteScript;

mod stub;
pub use stub::NoteStub;

mod vault;
pub use vault::NoteVault;

// CONSTANTS
// ================================================================================================

/// The depth of the Merkle tree used to commit to notes produced in a block.
pub const NOTE_TREE_DEPTH: u8 = 20;

/// The depth of the leafs in the note Merkle tree used to commit to notes produced in a block.
/// This is equal `NOTE_TREE_DEPTH + 1`. In the kernel we do not authenticate leaf data directly
/// but rather authenticate hash(left_leaf, right_leaf).
pub const NOTE_LEAF_DEPTH: u8 = NOTE_TREE_DEPTH + 1;

// NOTE
// ================================================================================================

/// A note which can be used to transfer assets between accounts.
///
/// This struct is a full description of a note which is needed to execute a note in a transaction.
/// A note consists of:
///
/// Core on-chain data which is used to execute a note:
/// - A script which must be executed in a context of some account to claim the assets.
/// - A set of inputs which are placed onto the stack before a note's script is executed.
/// - A set of assets stored in a vault.
/// - A serial number which can be used to break linkability between note hash and note nullifier.
///
/// Auxiliary data which is used to verify authenticity and signal additional information:
/// - A metadata object which contains information about the sender, the tag and the number of
///   assets in the note.
/// - A proof which provides the data required to authenticate the note against the note root of
///   the block in which the note was produced.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct Note {
    script: NoteScript,
    inputs: NoteInputs,
    vault: NoteVault,
    serial_num: Word,
    metadata: NoteMetadata,
    proof: Option<NoteInclusionProof>,
}

impl Note {
    // CONSTRUCTOR
    // --------------------------------------------------------------------------------------------
    /// Returns a new note created with the specified parameters.
    ///
    /// # Errors
    /// Returns an error if:
    /// - The number of inputs exceeds 16.
    /// - The number of provided assets exceeds 1000.
    /// - The list of assets contains duplicates.
    pub fn new(
        script: NoteScript,
        inputs: &[Felt],
        assets: &[Asset],
        serial_num: Word,
        sender: AccountId,
        tag: Felt,
        proof: Option<NoteInclusionProof>,
    ) -> Result<Self, NoteError> {
        let vault = NoteVault::new(assets)?;
        let num_assets = vault.num_assets();
        Ok(Self {
            script,
            inputs: NoteInputs::new(inputs),
            vault,
            serial_num,
            metadata: NoteMetadata::new(sender, tag, Felt::new(num_assets as u64)),
            proof,
        })
    }

    // PUBLIC ACCESSORS
    // --------------------------------------------------------------------------------------------

    /// Returns a reference script which locks the assets of this note.
    pub fn script(&self) -> &NoteScript {
        &self.script
    }

    /// Returns a reference to the note inputs.
    pub fn inputs(&self) -> &NoteInputs {
        &self.inputs
    }

    /// Returns a reference to the asset vault of this note.
    pub fn vault(&self) -> &NoteVault {
        &self.vault
    }

    /// Returns a serial number of this note.
    pub fn serial_num(&self) -> Word {
        self.serial_num
    }

    /// Returns the origin of the note.
    pub fn origin(&self) -> Option<&NoteOrigin> {
        self.proof.as_ref().map(|x| x.origin())
    }

    /// Returns the note inclusion proof.
    pub fn proof(&self) -> &Option<NoteInclusionProof> {
        &self.proof
    }

    /// Returns the metadata associated with this note.
    pub fn metadata(&self) -> &NoteMetadata {
        &self.metadata
    }

    /// Returns the recipient of this note.
    /// Recipient is defined and calculated as:
    ///  hash(hash(hash(serial_num, [0; 4]), script_hash), input_hash)
    pub fn recipient(&self) -> Digest {
        let serial_num_hash = Hasher::merge(&[self.serial_num.into(), Digest::default()]);
        let merge_script = Hasher::merge(&[serial_num_hash, self.script.hash()]);
        Hasher::merge(&[merge_script, self.inputs.hash()])
    }

    /// Returns a commitment to this note.
    ///
    /// The note hash is computed as:
    ///   hash(hash(hash(hash(serial_num, [0; 4]), script_hash), input_hash), vault_hash).
    /// This achieves the following properties:
    /// - Every note can be reduced to a single unique hash.
    /// - To compute a note's hash, we do not need to know the note's serial_num. Knowing the hash
    ///   of the serial_num (as well as script hash, input hash and note vault) is sufficient.
    /// - Moreover, we define `recipient` as:
    ///     `hash(hash(hash(serial_num, [0; 4]), script_hash), input_hash)`
    ///  This allows computing note hash from recipient and note vault.
    /// - We compute hash of serial_num as hash(serial_num, [0; 4]) to simplify processing within
    ///   the VM.
    pub fn hash(&self) -> Digest {
        let recipient = self.recipient();
        Hasher::merge(&[recipient, self.vault.hash()])
    }

    /// Returns the value used to authenticate a notes existence in the note tree. This is computed
    /// as a 2-to-1 hash of the note hash and note metadata [hash(note_hash, note_metadata)]
    pub fn authentication_hash(&self) -> Digest {
        Hasher::merge(&[self.hash(), Word::from(self.metadata()).into()])
    }

    /// Returns the nullifier for this note.
    ///
    /// The nullifier is computed as hash(serial_num, script_hash, input_hash, vault_hash).
    /// This achieves the following properties:
    /// - Every note can be reduced to a single unique nullifier.
    /// - We cannot derive a note's hash from its nullifier.
    /// - To compute the nullifier we must know all components of the note: serial_num,
    ///   script_hash, input_hash and vault hash.
    pub fn nullifier(&self) -> Digest {
        // The total number of elements to be hashed is 16. We can absorb them in
        // exactly two permutations
        let target_num_elements = 4 * WORD_SIZE;
        let mut elements: Vec<Felt> = Vec::with_capacity(target_num_elements);
        elements.extend_from_slice(&self.serial_num);
        elements.extend_from_slice(self.script.hash().as_elements());
        elements.extend_from_slice(self.inputs.hash().as_elements());
        elements.extend_from_slice(self.vault.hash().as_elements());
        Hasher::hash_elements(&elements)
    }

    // MODIFIERS
    // --------------------------------------------------------------------------------------------
    pub fn set_proof(&mut self, proof: NoteInclusionProof) {
        self.proof = Some(proof);
    }
}
