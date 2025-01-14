use super::{
    Account, AccountId, BlockHeader, ChainMmr, DataStoreError, ModuleAst, Note, NoteOrigin,
};

/// The [DataStore] trait defines the interface that transaction objects use to fetch data
/// required for transaction execution.
pub trait DataStore {
    /// Returns the [Account], [BlockHeader], [ChainMmr], and [Note]s required for transaction
    /// execution.
    fn get_transaction_data(
        &self,
        account_id: AccountId,
        block_num: u32,
        notes: &[NoteOrigin],
    ) -> Result<(Account, BlockHeader, ChainMmr, Vec<Note>), DataStoreError>;

    /// Returns the account code [ModuleAst] associated with the the specified [AccountId].
    fn get_account_code(&self, account_id: AccountId) -> Result<ModuleAst, DataStoreError>;
}
