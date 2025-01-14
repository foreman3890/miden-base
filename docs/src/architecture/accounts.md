# Accounts
Miden aims to support expressive smart contracts via a Turing-complete language. For smart contracts the go-to solution is account-based state. In Miden, an account is an entity which holds assets and defines rules of how these assets can be transferred. They are basic building blocks representing a user or an autonomous smart contract.

## Account Design
The diagram below illustrates basic components of an account. In Miden every account is a smart contract.

<p align="center">
    <img src="../diagrams/architecture/account/Account_Definition.png">
</p>

In the above picture, you can see:

* **Account ID &rarr;** a unique identifier of an account which does not change throughout its lifetime
* **Storage &rarr;** a user-defined data which can be stored in an account
* **Nonce &rarr;** a counter which must be incremented whenever account state changes
* **Vault &rarr;** a collection of assets stored in an account
* **Code &rarr;** a collection of functions which define an external interface for an account

### Account ID
~63 bits (1 Felt) long identifier for the account. The first three significant bits specify its type and the [storage mode](https://0xpolygonmiden.github.io/miden-base/architecture/accounts.html#account-storage-modes). There are four types of accounts in Miden:

| | Regular updatable account | Regular immutable account | Faucet for fungible assets | Faucet for non-fungible assets |
|---|---|---|---|---|
| **Description** | Will be used by most users for a wallet. Code specification and changes possible. | Will be used by most smart contracts. Once deployed code cannot be changed | Users can issue fungible assets and customize them. | Users can issue non-fungible assets and customize them. |
| **Code updatability** | yes | no | no | no |
| **Most significant bits** | `00` | `01` | `10` | `11` |

#### Example account ID (big-endian)

<p align="center">
    <img src="../diagrams/architecture/account/Account_ID.png">
</p>

*Note: Miden uses little-endian by default, but big_endian for better readability in the picture above.*

### Account Storage
User-defined data that can be stored in an account. `AccountStorage` is composed of two components. 

The first component is a simple sparse Merkle tree of depth `8` which is index addressable. This provides the user with `256` `Word` slots. 

Users requiring additional storage can use the second component a `MerkleStore`. It allows users to store any Merkle structures they need. The root of the Merkle structure can be stored as a leaf in the simple sparse Merkle tree. When `AccountStorage` is serialized it will check to see if any of the leafs in the simple sparse Merkle tree are Merkle roots of other Merkle structures. If any Merkle roots are found then the Merkle structures will be persisted in the `AccountStorage` `MerkleStore`.

### Nonce
Counter which must be incremented whenever the account state changes. Nonce values must be strictly monotonically increasing and can be incremented by any value smaller than 2^{32} for every account update.

### Vault
Asset container for an account.

An account vault can contain an unlimited number of [assets](https://0xpolygonmiden.github.io/miden-base/architecture/assets.html). The assets are stored in a sparse
Merkle tree as follows:

* For fungible assets, the index of a node is defined by the issuing faucet ID, and the value
  of the node is the asset itself. Thus, for any fungible asset there will be only one node
  in the tree.
* For non-fungible assets, the index is defined by the asset itself, and the asset is also
  the value of the node.

An account vault can be reduced to a single hash which is the root of the sparse Merkle tree.

### Code
Interface for accounts. In Miden every account is a smart contract. It has an interface that exposes functions that can be called by note scripts. Functions exposed by the account have the following properties:

* Functions are actually roots of [Miden program MASTs](https://wiki.polygon.technology/docs/miden/user_docs/assembly/main) (i.e., 32-byte hash). Thus, function identifier is a commitment to the code which is executed when a function is invoked.
* Only account functions have mutable access to an account's storage and vault. Therefore, the only way to modify an account's internal state is through one of account's functions.
* Account functions can take parameters and can create new notes.

*Note: Since code in Miden is expresed as MAST, every function is a commitment to the underlying code. The code cannot change unnoticed to the user because its hash would change. Behind any MAST root there can only be `256` functions*

## Account creation
For an account to exist it must be present in the [Account DB](https://0xpolygonmiden.github.io/miden-base/architecture/state.html#account-database) kept by the Miden Node(s). However, new accounts can be created locally by users using a wallet.

The process is as follows:

* Alice grinds a new Account ID (according to the account types) using a wallet
* Alice's Miden Client requests the Miden Node to check if new Account ID already exists
* Alice shares the new Account ID to Bob (eg. when Alice wants to receive funds)
* Bob executes a transaction and creates a note that contains an asset for Alice
* Alice consumes Bob's note to receive the asset in a transaction
* Depending on the account storage mode (private vs. public) and transaction type (local vs. network) the Operator receives new Account ID eventually and - if transaction is correct - adds the ID to the Account DB

## Account storage modes
Account data - stored by the Miden Node - can be public, private, or encrypted. The third and fourth most significant bits of the account ID specifies whether the account data is public `00`, encrypted `01`, or private `11`.

* Accounts with **public state**, where the actual state is stored onchain. These would be similar to how accounts work in public blockchains. Smart contracts that depend on public shared state should be stored public on Miden, e.g., DEX contract.
* Account with **encrypted state**, where the account data is stored onchain but in encrypted text. It provides liveness guarantee of the protocol for the account in question.  
* Accounts with **private state**, where only the hash of the account is stored onchain. Users who want stay private and take care of their own data should choose this mode. The hash is defined as: `hash([account ID, 0, 0, nonce], [vault root], [storage root], [code root])`. 
