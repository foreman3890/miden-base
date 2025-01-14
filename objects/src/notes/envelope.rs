use super::{Digest, Felt, NoteMetadata, Vec, Word};
use miden_core::StarkField;

// NOTE ENVELOPE
// ================================================================================================

/// Holds information that is relevant to the recipient of a note.
/// Contains:
/// - note_hash: hash of the note that was created
/// - note_metadata: metadata of the note that was created. Metadata is padded with ZERO such that
///   it is four elements in size (a word). The metadata includes the following elements:
///     - sender
///     - tag
///     - ZERO
///     - ZERO
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct NoteEnvelope {
    note_hash: Digest,
    note_metadata: NoteMetadata,
}

impl NoteEnvelope {
    /// Creates a new [NoteEnvelope] object.
    pub fn new(note_hash: Digest, note_metadata: NoteMetadata) -> Self {
        Self {
            note_hash,
            note_metadata,
        }
    }

    /// Returns the hash of the note that was created.
    pub fn note_hash(&self) -> Digest {
        self.note_hash
    }

    /// Returns the metadata of the note that was created.
    pub fn metadata(&self) -> &NoteMetadata {
        &self.note_metadata
    }
}

impl From<NoteEnvelope> for [Felt; 8] {
    fn from(cni: NoteEnvelope) -> Self {
        let mut elements: [Felt; 8] = Default::default();
        elements[..4].copy_from_slice(cni.note_hash.as_elements());
        elements[4..].copy_from_slice(&Word::from(cni.metadata()));
        elements
    }
}

impl From<NoteEnvelope> for [Word; 2] {
    fn from(cni: NoteEnvelope) -> Self {
        let mut elements: [Word; 2] = Default::default();
        elements[0].copy_from_slice(cni.note_hash.as_elements());
        elements[1].copy_from_slice(&Word::from(cni.metadata()));
        elements
    }
}

impl From<NoteEnvelope> for [u8; 64] {
    fn from(cni: NoteEnvelope) -> Self {
        let mut elements: [u8; 64] = [0; 64];
        let note_metadata_bytes = Word::from(cni.metadata())
            .iter()
            .flat_map(|x| x.as_int().to_le_bytes())
            .collect::<Vec<u8>>();
        elements[..32].copy_from_slice(&cni.note_hash.as_bytes());
        elements[32..].copy_from_slice(&note_metadata_bytes);
        elements
    }
}
