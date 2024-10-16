use std::str::FromStr;
use semver::Version;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use sologger_log_context::programs_selector::ProgramsSelector;
use sologger_log_context::sologger_log_context::LogContext;
use thiserror::Error;

/// Extracts log messages from a Response<RpcLogsResponse> and returns a vector of LogContexts
pub fn from_rpc_response(
    response: &Response<RpcLogsResponse>,
    program_selector: &ProgramsSelector,
) -> anyhow::Result<Vec<LogContext>> {
    let transaction_error = match response.value.err.clone() {
        None => "".to_string(),
        Some(err) => {
            format!("{}", err)
        }
    };

    let sig = response.value.signature.to_string();
    let log_contexts = LogContext::parse_logs(
        &response.value.logs,
        transaction_error,
        program_selector,
        response.context.slot,
        sig,
    );

    Ok(log_contexts)
}

/// Extracts log messages from a RpcLogsResponse and returns a vector of LogContexts
pub fn from_rpc_logs_response(
    rpc_logs_response: &RpcLogsResponse,
    slot: u64,
    program_selector: &ProgramsSelector,
) -> anyhow::Result<Vec<LogContext>> {
    let transaction_error = match rpc_logs_response.err.clone() {
        None => "".to_string(),
        Some(err) => {
            format!("{}", err)
        }
    };

    let sig = rpc_logs_response.signature.to_string();
    let log_contexts = LogContext::parse_logs(
        &rpc_logs_response.logs,
        transaction_error,
        program_selector,
        slot,
        sig,
    );

    Ok(log_contexts)
}

// The following code is copied from Solana libraries. Including the Solana libraries to support these structs and enums is not WASM friendly.

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RpcLogsResponse {
    pub signature: String, // Signature as base58 string
    pub err: Option<TransactionError>,
    pub logs: Vec<String>,
}

/// Reasons a transaction might be rejected.
#[cfg_attr(feature = "frozen-abi", derive(AbiExample, AbiEnumVisitor))]
#[derive(Error, Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub enum TransactionError {
    /// An account is already being processed in another transaction in a way
    /// that does not support parallelism
    #[error("Account in use")]
    AccountInUse,

    /// A `Pubkey` appears twice in the transaction's `account_keys`.  Instructions can reference
    /// `Pubkey`s more than once but the message must contain a list with no duplicate keys
    #[error("Account loaded twice")]
    AccountLoadedTwice,

    /// Attempt to debit an account but found no record of a prior credit.
    #[error("Attempt to debit an account but found no record of a prior credit.")]
    AccountNotFound,

    /// Attempt to load a program that does not exist
    #[error("Attempt to load a program that does not exist")]
    ProgramAccountNotFound,

    /// The from `Pubkey` does not have sufficient balance to pay the fee to schedule the transaction
    #[error("Insufficient funds for fee")]
    InsufficientFundsForFee,

    /// This account may not be used to pay transaction fees
    #[error("This account may not be used to pay transaction fees")]
    InvalidAccountForFee,

    /// The bank has seen this transaction before. This can occur under normal operation
    /// when a UDP packet is duplicated, as a user error from a client not updating
    /// its `recent_blockhash`, or as a double-spend attack.
    #[error("This transaction has already been processed")]
    AlreadyProcessed,

    /// The bank has not seen the given `recent_blockhash` or the transaction is too old and
    /// the `recent_blockhash` has been discarded.
    #[error("Blockhash not found")]
    BlockhashNotFound,

    /// An error occurred while processing an instruction. The first element of the tuple
    /// indicates the instruction index in which the error occurred.
    #[error("Error processing Instruction {0}: {1}")]
    InstructionError(u8, InstructionError),

    /// Loader call chain is too deep
    #[error("Loader call chain is too deep")]
    CallChainTooDeep,

    /// Transaction requires a fee but has no signature present
    #[error("Transaction requires a fee but has no signature present")]
    MissingSignatureForFee,

    /// Transaction contains an invalid account reference
    #[error("Transaction contains an invalid account reference")]
    InvalidAccountIndex,

    /// Transaction did not pass signature verification
    #[error("Transaction did not pass signature verification")]
    SignatureFailure,

    /// This program may not be used for executing instructions
    #[error("This program may not be used for executing instructions")]
    InvalidProgramForExecution,

    /// Transaction failed to sanitize accounts offsets correctly
    /// implies that account locks are not taken for this TX, and should
    /// not be unlocked.
    #[error("Transaction failed to sanitize accounts offsets correctly")]
    SanitizeFailure,

    #[error("Transactions are currently disabled due to cluster maintenance")]
    ClusterMaintenance,

    /// Transaction processing left an account with an outstanding borrowed reference
    #[error("Transaction processing left an account with an outstanding borrowed reference")]
    AccountBorrowOutstanding,

    /// Transaction would exceed max Block Cost Limit
    #[error("Transaction would exceed max Block Cost Limit")]
    WouldExceedMaxBlockCostLimit,

    /// Transaction version is unsupported
    #[error("Transaction version is unsupported")]
    UnsupportedVersion,

    /// Transaction loads a writable account that cannot be written
    #[error("Transaction loads a writable account that cannot be written")]
    InvalidWritableAccount,

    /// Transaction would exceed max account limit within the block
    #[error("Transaction would exceed max account limit within the block")]
    WouldExceedMaxAccountCostLimit,

    /// Transaction would exceed account data limit within the block
    #[error("Transaction would exceed account data limit within the block")]
    WouldExceedAccountDataBlockLimit,

    /// Transaction locked too many accounts
    #[error("Transaction locked too many accounts")]
    TooManyAccountLocks,

    /// Address lookup table not found
    #[error("Transaction loads an address table account that doesn't exist")]
    AddressLookupTableNotFound,

    /// Attempted to lookup addresses from an account owned by the wrong program
    #[error("Transaction loads an address table account with an invalid owner")]
    InvalidAddressLookupTableOwner,

    /// Attempted to lookup addresses from an invalid account
    #[error("Transaction loads an address table account with invalid data")]
    InvalidAddressLookupTableData,

    /// Address table lookup uses an invalid index
    #[error("Transaction address table lookup uses an invalid index")]
    InvalidAddressLookupTableIndex,

    /// Transaction leaves an account with a lower balance than rent-exempt minimum
    #[error("Transaction leaves an account with a lower balance than rent-exempt minimum")]
    InvalidRentPayingAccount,

    /// Transaction would exceed max Vote Cost Limit
    #[error("Transaction would exceed max Vote Cost Limit")]
    WouldExceedMaxVoteCostLimit,

    /// Transaction would exceed total account data limit
    #[error("Transaction would exceed total account data limit")]
    WouldExceedAccountDataTotalLimit,

    /// Transaction contains a duplicate instruction that is not allowed
    #[error("Transaction contains a duplicate instruction ({0}) that is not allowed")]
    DuplicateInstruction(u8),

    /// Transaction results in an account with insufficient funds for rent
    #[error(
        "Transaction results in an account ({account_index}) with insufficient funds for rent"
    )]
    InsufficientFundsForRent { account_index: u8 },

    /// Transaction exceeded max loaded accounts data size cap
    #[error("Transaction exceeded max loaded accounts data size cap")]
    MaxLoadedAccountsDataSizeExceeded,

    /// LoadedAccountsDataSizeLimit set for transaction must be greater than 0.
    #[error("LoadedAccountsDataSizeLimit set for transaction must be greater than 0.")]
    InvalidLoadedAccountsDataSizeLimit,

    /// Sanitized transaction differed before/after feature activiation. Needs to be resanitized.
    #[error("ResanitizationNeeded")]
    ResanitizationNeeded,

    /// Program execution is temporarily restricted on an account.
    #[error("Execution of the program referenced by account at index {account_index} is temporarily restricted.")]
    ProgramExecutionTemporarilyRestricted { account_index: u8 },

    /// The total balance before the transaction does not equal the total balance after the transaction
    #[error("Sum of account balances before and after transaction do not match")]
    UnbalancedTransaction,

    /// Program cache hit max limit.
    #[error("Program cache hit max limit")]
    ProgramCacheHitMaxLimit,
}

impl From<SanitizeError> for TransactionError {
    fn from(_: SanitizeError) -> Self {
        Self::SanitizeFailure
    }
}

impl From<SanitizeMessageError> for TransactionError {
    fn from(err: SanitizeMessageError) -> Self {
        match err {
            SanitizeMessageError::AddressLoaderError(err) => Self::from(err),
            _ => Self::SanitizeFailure,
        }
    }
}


#[derive(PartialEq, Debug, Error, Eq, Clone)]
pub enum SanitizeMessageError {
    #[error("index out of bounds")]
    IndexOutOfBounds,
    #[error("value out of bounds")]
    ValueOutOfBounds,
    #[error("invalid value")]
    InvalidValue,
    #[error("{0}")]
    AddressLoaderError(#[from] AddressLoaderError),
}

impl From<SanitizeError> for SanitizeMessageError {
    fn from(err: SanitizeError) -> Self {
        match err {
            SanitizeError::IndexOutOfBounds => Self::IndexOutOfBounds,
            SanitizeError::ValueOutOfBounds => Self::ValueOutOfBounds,
            SanitizeError::InvalidValue => Self::InvalidValue,
        }
    }
}

impl From<AddressLoaderError> for TransactionError {
    fn from(err: AddressLoaderError) -> Self {
        match err {
            AddressLoaderError::Disabled => Self::UnsupportedVersion,
            AddressLoaderError::SlotHashesSysvarNotFound => Self::AccountNotFound,
            AddressLoaderError::LookupTableAccountNotFound => Self::AddressLookupTableNotFound,
            AddressLoaderError::InvalidAccountOwner => Self::InvalidAddressLookupTableOwner,
            AddressLoaderError::InvalidAccountData => Self::InvalidAddressLookupTableData,
            AddressLoaderError::InvalidLookupIndex => Self::InvalidAddressLookupTableIndex,
        }
    }
}

#[derive(Debug, Error, PartialEq, Eq, Clone)]
pub enum AddressLoaderError {
    /// Address loading from lookup tables is disabled
    #[error("Address loading from lookup tables is disabled")]
    Disabled,

    /// Failed to load slot hashes sysvar
    #[error("Failed to load slot hashes sysvar")]
    SlotHashesSysvarNotFound,

    /// Attempted to lookup addresses from a table that does not exist
    #[error("Attempted to lookup addresses from a table that does not exist")]
    LookupTableAccountNotFound,

    /// Attempted to lookup addresses from an account owned by the wrong program
    #[error("Attempted to lookup addresses from an account owned by the wrong program")]
    InvalidAccountOwner,

    /// Attempted to lookup addresses from an invalid account
    #[error("Attempted to lookup addresses from an invalid account")]
    InvalidAccountData,

    /// Address lookup contains an invalid index
    #[error("Address lookup contains an invalid index")]
    InvalidLookupIndex,
}


#[derive(PartialEq, Debug, Error, Eq, Clone)]
pub enum SanitizeError {
    #[error("index out of bounds")]
    IndexOutOfBounds,
    #[error("value out of bounds")]
    ValueOutOfBounds,
    #[error("invalid value")]
    InvalidValue,
}

#[cfg_attr(feature = "frozen-abi", derive(AbiExample, AbiEnumVisitor))]
#[derive(Serialize, Deserialize, Debug, Error, PartialEq, Eq, Clone)]
pub enum InstructionError {
    /// Deprecated! Use CustomError instead!
    /// The program instruction returned an error
    #[error("generic instruction error")]
    GenericError,

    /// The arguments provided to a program were invalid
    #[error("invalid program argument")]
    InvalidArgument,

    /// An instruction's data contents were invalid
    #[error("invalid instruction data")]
    InvalidInstructionData,

    /// An account's data contents was invalid
    #[error("invalid account data for instruction")]
    InvalidAccountData,

    /// An account's data was too small
    #[error("account data too small for instruction")]
    AccountDataTooSmall,

    /// An account's balance was too small to complete the instruction
    #[error("insufficient funds for instruction")]
    InsufficientFunds,

    /// The account did not have the expected program id
    #[error("incorrect program id for instruction")]
    IncorrectProgramId,

    /// A signature was required but not found
    #[error("missing required signature for instruction")]
    MissingRequiredSignature,

    /// An initialize instruction was sent to an account that has already been initialized.
    #[error("instruction requires an uninitialized account")]
    AccountAlreadyInitialized,

    /// An attempt to operate on an account that hasn't been initialized.
    #[error("instruction requires an initialized account")]
    UninitializedAccount,

    /// Program's instruction lamport balance does not equal the balance after the instruction
    #[error("sum of account balances before and after instruction do not match")]
    UnbalancedInstruction,

    /// Program illegally modified an account's program id
    #[error("instruction illegally modified the program id of an account")]
    ModifiedProgramId,

    /// Program spent the lamports of an account that doesn't belong to it
    #[error("instruction spent from the balance of an account it does not own")]
    ExternalAccountLamportSpend,

    /// Program modified the data of an account that doesn't belong to it
    #[error("instruction modified data of an account it does not own")]
    ExternalAccountDataModified,

    /// Read-only account's lamports modified
    #[error("instruction changed the balance of a read-only account")]
    ReadonlyLamportChange,

    /// Read-only account's data was modified
    #[error("instruction modified data of a read-only account")]
    ReadonlyDataModified,

    /// An account was referenced more than once in a single instruction
    // Deprecated, instructions can now contain duplicate accounts
    #[error("instruction contains duplicate accounts")]
    DuplicateAccountIndex,

    /// Executable bit on account changed, but shouldn't have
    #[error("instruction changed executable bit of an account")]
    ExecutableModified,

    /// Rent_epoch account changed, but shouldn't have
    #[error("instruction modified rent epoch of an account")]
    RentEpochModified,

    /// The instruction expected additional account keys
    #[error("insufficient account keys for instruction")]
    NotEnoughAccountKeys,

    /// Program other than the account's owner changed the size of the account data
    #[error("program other than the account's owner changed the size of the account data")]
    AccountDataSizeChanged,

    /// The instruction expected an executable account
    #[error("instruction expected an executable account")]
    AccountNotExecutable,

    /// Failed to borrow a reference to account data, already borrowed
    #[error("instruction tries to borrow reference for an account which is already borrowed")]
    AccountBorrowFailed,

    /// Account data has an outstanding reference after a program's execution
    #[error("instruction left account with an outstanding borrowed reference")]
    AccountBorrowOutstanding,

    /// The same account was multiply passed to an on-chain program's entrypoint, but the program
    /// modified them differently.  A program can only modify one instance of the account because
    /// the runtime cannot determine which changes to pick or how to merge them if both are modified
    #[error("instruction modifications of multiply-passed account differ")]
    DuplicateAccountOutOfSync,

    /// Allows on-chain programs to implement program-specific error types and see them returned
    /// by the Solana runtime. A program-specific error may be any type that is represented as
    /// or serialized to a u32 integer.
    #[error("custom program error: {0:#x}")]
    Custom(u32),

    /// The return value from the program was invalid.  Valid errors are either a defined builtin
    /// error value or a user-defined error in the lower 32 bits.
    #[error("program returned invalid error code")]
    InvalidError,

    /// Executable account's data was modified
    #[error("instruction changed executable accounts data")]
    ExecutableDataModified,

    /// Executable account's lamports modified
    #[error("instruction changed the balance of an executable account")]
    ExecutableLamportChange,

    /// Executable accounts must be rent exempt
    #[error("executable accounts must be rent exempt")]
    ExecutableAccountNotRentExempt,

    /// Unsupported program id
    #[error("Unsupported program id")]
    UnsupportedProgramId,

    /// Cross-program invocation call depth too deep
    #[error("Cross-program invocation call depth too deep")]
    CallDepth,

    /// An account required by the instruction is missing
    #[error("An account required by the instruction is missing")]
    MissingAccount,

    /// Cross-program invocation reentrancy not allowed for this instruction
    #[error("Cross-program invocation reentrancy not allowed for this instruction")]
    ReentrancyNotAllowed,

    /// Length of the seed is too long for address generation
    #[error("Length of the seed is too long for address generation")]
    MaxSeedLengthExceeded,

    /// Provided seeds do not result in a valid address
    #[error("Provided seeds do not result in a valid address")]
    InvalidSeeds,

    /// Failed to reallocate account data of this length
    #[error("Failed to reallocate account data")]
    InvalidRealloc,

    /// Computational budget exceeded
    #[error("Computational budget exceeded")]
    ComputationalBudgetExceeded,

    /// Cross-program invocation with unauthorized signer or writable account
    #[error("Cross-program invocation with unauthorized signer or writable account")]
    PrivilegeEscalation,

    /// Failed to create program execution environment
    #[error("Failed to create program execution environment")]
    ProgramEnvironmentSetupFailure,

    /// Program failed to complete
    #[error("Program failed to complete")]
    ProgramFailedToComplete,

    /// Program failed to compile
    #[error("Program failed to compile")]
    ProgramFailedToCompile,

    /// Account is immutable
    #[error("Account is immutable")]
    Immutable,

    /// Incorrect authority provided
    #[error("Incorrect authority provided")]
    IncorrectAuthority,

    /// Failed to serialize or deserialize account data
    ///
    /// Warning: This error should never be emitted by the runtime.
    ///
    /// This error includes strings from the underlying 3rd party Borsh crate
    /// which can be dangerous because the error strings could change across
    /// Borsh versions. Only programs can use this error because they are
    /// consistent across Solana software versions.
    ///
    #[error("Failed to serialize or deserialize account data: {0}")]
    BorshIoError(String),

    /// An account does not have enough lamports to be rent-exempt
    #[error("An account does not have enough lamports to be rent-exempt")]
    AccountNotRentExempt,

    /// Invalid account owner
    #[error("Invalid account owner")]
    InvalidAccountOwner,

    /// Program arithmetic overflowed
    #[error("Program arithmetic overflowed")]
    ArithmeticOverflow,

    /// Unsupported sysvar
    #[error("Unsupported sysvar")]
    UnsupportedSysvar,

    /// Illegal account owner
    #[error("Provided owner is not allowed")]
    IllegalOwner,

    /// Accounts data allocations exceeded the maximum allowed per transaction
    #[error("Accounts data allocations exceeded the maximum allowed per transaction")]
    MaxAccountsDataAllocationsExceeded,

    /// Max accounts exceeded
    #[error("Max accounts exceeded")]
    MaxAccountsExceeded,

    /// Max instruction trace length exceeded
    #[error("Max instruction trace length exceeded")]
    MaxInstructionTraceLengthExceeded,

    /// Builtin programs must consume compute units
    #[error("Builtin programs must consume compute units")]
    BuiltinProgramsMustConsumeComputeUnits,
    // Note: For any new error added here an equivalent ProgramError and its
    // conversions must also be added
}

pub type Slot = u64;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RpcResponseContext {
    pub slot: Slot,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_version: Option<RpcApiVersion>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RpcApiVersion(semver::Version);

impl std::ops::Deref for RpcApiVersion {
    type Target = semver::Version;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Default for RpcApiVersion {
    fn default() -> Self {
        Self(Version {
            major: env!("CARGO_PKG_VERSION_MAJOR").parse().unwrap(),
            minor: env!("CARGO_PKG_VERSION_MINOR").parse().unwrap(),
            patch: env!("CARGO_PKG_VERSION_PATCH").parse().unwrap(),
            pre: Default::default(),
            build: Default::default(),
        })
    }
}

impl Serialize for RpcApiVersion {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for RpcApiVersion {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: String = Deserialize::deserialize(deserializer)?;
        Ok(RpcApiVersion(
            semver::Version::from_str(&s).map_err(serde::de::Error::custom)?,
        ))
    }
}

impl RpcResponseContext {
    pub fn new(slot: Slot) -> Self {
        Self {
            slot,
            api_version: Some(RpcApiVersion::default()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Response<T> {
    pub context: RpcResponseContext,
    pub value: T,
}

// End of Solana copied code


#[cfg(test)]
mod tests {
    use sologger_log_context::programs_selector::ProgramsSelector;
    use crate::log_context_transformer_wasm::{from_rpc_logs_response, from_rpc_response, Response, RpcLogsResponse, RpcResponseContext};

    #[test]
    pub fn test_parse_rpc_logs_response() {
        let rpc_logs_response = RpcLogsResponse {
            signature: "pF5oPR8R4vJwU2KeQm8BAAGYcTiikZkpJAmP8TuuVztkL2K6wZhxVKy9t6jSCMSpMMD3VE6Qek1YL5JAFvuBLQw".to_string(),
            err: None,
            logs: vec!["Program 11111111111111111111111111111111 invoke [1]".to_string(), "Program 11111111111111111111111111111111 success".to_string()],
        };
    
        let logs_contexts = from_rpc_logs_response(
            &rpc_logs_response,
            323432,
            &ProgramsSelector::new_all_programs(),
        )
        .unwrap();
    
        assert_eq!(logs_contexts.len(), 1);
    }

    #[test]
    pub fn test_parse_rpc_response() {
        let rpc_logs_response = RpcLogsResponse {
            signature: "pF5oPR8R4vJwU2KeQm8BAAGYcTiikZkpJAmP8TuuVztkL2K6wZhxVKy9t6jSCMSpMMD3VE6Qek1YL5JAFvuBLQw".to_string(),
            err: None,
            logs: vec!["Program 11111111111111111111111111111111 invoke [1]".to_string(), "Program 11111111111111111111111111111111 success".to_string()],
        };
    
        let response = Response {
            context: RpcResponseContext {
                slot: 12324,
                api_version: None,
            },
            value: rpc_logs_response,
        };
    
        let logs_contexts =
            from_rpc_response(&response, &ProgramsSelector::new_all_programs()).unwrap();
    
        assert_eq!(logs_contexts.len(), 1);
    }
    
}
