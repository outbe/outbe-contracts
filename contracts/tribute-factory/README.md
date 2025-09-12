# Tribute Factory Smart Contract

## Overview

The Tribute Factory smart contract is a critical component of the ecosystem that serves as the secure gateway for creating Tribute NFTs from L2 network data. It acts as the bridge between Layer 2 consumption data and Layer 1 tribute representation, implementing sophisticated cryptographic protocols including Trusted Execution Environment (TEE) integration, zero-knowledge proof verification, and end-to-end encryption.

The contract enables users to securely "offer" their consumption tributes by processing encrypted tribute data, validating zero-knowledge proofs, and minting non-fungible, non-transferable tribute tokens that represent consumption for specific worldwide days.

## Key Concepts

### Tribute Creation Flow

- **Tribute Draft**: Pre-stage consumption aggregates created on L2 from Consumption Units (CUs)
- **Tribute Offering**: The process of submitting encrypted tribute data to create L1 Tributes
- **TEE Integration**: Trusted Execution Environment for secure data decryption and obfuscation
- **ZK Proof Verification**: Cryptographic validation of tribute data integrity using PlonK algorithm

### Security Architecture

- **ECDHE Encryption**: Elliptic Curve Diffie-Hellman Ephemeral key exchange for data encryption
- **ChaCha20-Poly1305**: Authenticated encryption for tribute payload protection
- **Data Obfuscation**: TEE applies salts and hashing to break L2-L1 data linkage
- **Unique ID Tracking**: Prevents duplicate tribute submissions and CU hash reuse

### Cryptographic Components

- **X25519 Key Exchange**: For secure shared secret generation
- **Blake3 Hashing**: For tribute ID and hash generation
- **HKDF Key Derivation**: Hierarchical key derivation for encryption keys
- **ZK Proof System**: PlonK-based zero-knowledge proof verification

## Business Logic

### Tribute Input Structure

Each tribute submission contains:
- **Tribute Draft ID**: Unique identifier derived from owner and worldwide day
- **Owner**: Derivative L2 address based on Blake3 hashing  
- **Worldwide Day**: ISO 8601 date for consumption period
- **Settlement Data**: Currency and amounts (base + atto precision)
- **Nominal Data**: Quantities in natural and fractional units
- **CU Hashes**: Vector of consumption unit hashes from L2

### Encryption and Decryption Flow

1. **Key Exchange**: Wallet App and TEE perform ECDHE key exchange
2. **Data Encryption**: Tribute payload encrypted using ChaCha20-Poly1305
3. **Secure Transmission**: Cipher text, nonce, and ephemeral public key sent to contract
4. **TEE Decryption**: Contract uses TEE to decrypt tribute data
5. **Data Obfuscation**: TEE applies salts and hashing for privacy protection
6. **Tribute Minting**: Validated data used to mint Tribute NFT

### Zero-Knowledge Proof Validation

The contract verifies ZK proofs that demonstrate:
- **Data Integrity**: Tribute hash computation correctness
- **Merkle Tree Inclusion**: Proof that tribute exists in L2 state
- **Owner Authorization**: Verification that owner can be derived from public key
- **Settlement Accuracy**: Validation of consumption data authenticity

### Data Protection Mechanisms

1. **Tribute ID Obfuscation**:
   ```
   tribute_id = blake3(prefix | tribute_draft_id | salt)
   ```

2. **CU Hash Protection**:
   ```
   protected_cu_hash = blake3(prefix | cu_hash | salt)
   ```

3. **Unique Constraints**:
   - One tribute per owner per worldwide day
   - Unique tribute IDs across all submissions
   - No CU hash reuse across different tributes

## Technical Architecture

### State Management

- **Config**: Contract configuration with tribute address and TEE setup
- **TeeConfig**: Cryptographic keys and salt for secure operations
- **UsedTributeIds**: Tracking to prevent duplicate tribute submissions
- **UsedCuHashes**: Tracking to prevent consumption unit reuse
- **Owner**: Contract ownership and access control

### External Dependencies

- **Tribute Contract**: Target contract for minting Tribute NFTs
- **TEE Module**: Trusted execution environment for data processing
- **Price Oracle**: Exchange rates for tribute value calculations
- **L2 Network**: Source of consumption data and Merkle proofs

## API Reference

### Messages

#### InstantiateMsg
```rust
pub struct InstantiateMsg {
    pub tribute_address: Option<Addr>,    // Target tribute contract
    pub owner: Option<Addr>,              // Contract owner
    pub tee_config: Option<TeeSetup>,     // TEE configuration
    pub zk_config: Option<ZkSetup>,       // ZK proof setup
}
```

#### TeeSetup
```rust
pub struct TeeSetup {
    pub private_key: Base58Binary,    // X25519 private key for ECDHE
    pub salt: Base58Binary,           // Salt for hashing operations
}
```

#### ExecuteMsg
```rust
pub enum ExecuteMsg {
    UpdateConfig {
        new_owner: Option<Addr>,
        new_tribute_address: Option<Addr>,
        new_tee_config: Option<TeeSetup>,
    },
    
    /// Secure tribute offering with encryption
    Offer {
        cipher_text: Base58Binary,        // Encrypted tribute data
        nonce: Base58Binary,              // Encryption nonce
        ephemeral_pubkey: Base58Binary,   // ECDHE ephemeral public key
        zk_proof: ZkProof,                // Zero-knowledge proof
        tribute_owner_l1: Option<Addr>,   // L1 owner (demo feature)
    },
    
    /// Insecure offering for testing
    OfferInsecure {
        tribute_input: TributeInputPayload,
        zk_proof: ZkProof,
        tribute_owner_l1: Option<Addr>,
    },
    
    BurnAll {},  // Demo cleanup function
}
```

#### ZkProof Structure
```rust
pub struct ZkProof {
    pub proof: Base58Binary,              // PlonK proof data
    pub public_data: ZkProofPublicData,   // Public proof inputs
    pub verification_key: Base58Binary,   // Verification key
}

pub struct ZkProofPublicData {
    pub public_key: Base58Binary,     // User's public key
    pub merkle_root: Base58Binary,    // L2 state Merkle root
}
```

### Core Operations

#### Secure Tribute Offering
1. **Decrypt**: Use TEE to decrypt tribute payload
2. **Validate**: Verify ZK proof and data integrity
3. **Obfuscate**: Apply TEE data protection mechanisms
4. **Check Uniqueness**: Ensure no duplicate submissions
5. **Mint Tribute**: Create NFT through Tribute contract

#### Configuration Management
- Update contract owner and permissions
- Modify TEE configuration for key rotation
- Change target tribute contract address
- Update ZK verification parameters

## Cryptographic Implementation

### ECDHE Key Exchange
```rust
// Generate shared secret using X25519
let shared_secret = private_key.diffie_hellman(&ephemeral_public_key);

// Derive encryption key using HKDF
let encryption_key = hkdf_expand(shared_secret, salt, "tribute-factory-encryption");
```

### ChaCha20-Poly1305 Decryption
```rust
let cipher = ChaCha20Poly1305::new(&encryption_key);
let plaintext = cipher.decrypt(&nonce, cipher_text.as_slice())?;
let tribute_input: TributeInputPayload = from_json(&plaintext)?;
```

### Hash Generation Algorithms
```rust
// Tribute Draft ID
let tribute_draft_id = blake3::hash(owner.as_bytes() | worldwide_day.as_bytes());

// Tribute Hash for Merkle Tree
let tribute_hash = blake3::hash(
    tribute_draft_id | settlement_currency | settlement_amounts | 
    nominal_quantities | cu_hashes
);
```

## Usage Examples

### Contract Instantiation
```rust
let msg = InstantiateMsg {
    tribute_address: Some(Addr::unchecked("tribute_contract")),
    owner: Some(Addr::unchecked("factory_owner")),
    tee_config: Some(TeeSetup {
        private_key: Base58Binary::from("private_key_base58"),
        salt: Base58Binary::from("random_salt_base58"),
    }),
    zk_config: Some(ZkSetup {
        circuit: Base58Binary::from("zk_circuit_data"),
    }),
};
```

### Secure Tribute Submission
```rust
let offer_msg = ExecuteMsg::Offer {
    cipher_text: Base58Binary::from("encrypted_tribute_data"),
    nonce: Base58Binary::from("encryption_nonce"),
    ephemeral_pubkey: Base58Binary::from("ephemeral_public_key"),
    zk_proof: ZkProof {
        proof: Base58Binary::from("plonk_proof_data"),
        public_data: ZkProofPublicData {
            public_key: Base58Binary::from("user_public_key"),
            merkle_root: Base58Binary::from("l2_merkle_root"),
        },
        verification_key: Base58Binary::from("zk_verification_key"),
    },
    tribute_owner_l1: None,
};
```

### Testing with Insecure Mode
```rust
let test_msg = ExecuteMsg::OfferInsecure {
    tribute_input: TributeInputPayload {
        tribute_draft_id: Base58Binary::from("draft_id"),
        owner: Base58Binary::from("l2_owner_address"),
        worldwide_day: Iso8601Date::from_str("2025-06-10")?,
        settlement_currency: "USD".to_string(),
        settlement_base_amount: Uint64::new(100),
        settlement_atto_amount: Uint128::new(500000000000000000),
        nominal_base_qty: Uint64::new(50),
        nominal_atto_qty: Uint128::new(750000000000000000),
        cu_hashes: vec![Base58Binary::from("cu_hash_1")],
    },
    zk_proof: test_zk_proof,
    tribute_owner_l1: Some(Addr::unchecked("test_owner")),
};
```

## Deployment

### Prerequisites

- **Tribute Contract**: Deployed and operational NFT contract
- **TEE Module**: Trusted execution environment integration
- **ZK Circuit**: Compiled zero-knowledge proof circuit
- **Price Oracle**: For settlement value validation (if required)

### Configuration Steps

1. **Deploy Contract**: With initial TEE and ZK configurations
2. **Configure Keys**: Set up X25519 keypairs for ECDHE
3. **Verify TEE**: Test encryption/decryption functionality
4. **Validate ZK**: Test proof generation and verification
5. **Production Setup**: Deploy with secure key management

### Security Considerations

1. **Key Management**: Secure storage of TEE private keys
2. **Salt Security**: Use cryptographically secure random salts
3. **Access Control**: Proper owner management and permissions
4. **Audit Requirements**: Regular security audits of cryptographic implementations
5. **TEE Trust**: Verify TEE module integrity and security

## Testing

The contract includes comprehensive tests covering:

- **ECDHE Implementation**: Key exchange and encryption/decryption
- **ZK Proof Validation**: Proof verification algorithms
- **Unique ID Generation**: Tribute and CU hash uniqueness
- **Data Obfuscation**: TEE privacy protection mechanisms
- **Error Handling**: Invalid inputs and security violations

Run tests using:
```bash
cargo test
```

### Key Test Cases
```rust
#[test]
fn test_decrypt_tribute_input_with_hkdf() {
    // Test ECDHE key exchange and decryption
}

#[test]
fn test_unique_tribute_draft_id() {
    // Test tribute ID uniqueness constraints
}

#[test]
fn test_tee_config_validation() {
    // Test TEE configuration validation
}

#[test]
fn test_zk_proof_verification() {
    // Test zero-knowledge proof validation
}
```

## Integration Notes

### L2 to L1 Workflow

1. **L2 Aggregation**: Consumption Units aggregated into Tribute Drafts
2. **Wallet Encryption**: User encrypts tribute data using ECDHE
3. **ZK Proof Generation**: Create proof of L2 data integrity
4. **L1 Submission**: Submit encrypted data and proofs to Factory
5. **TEE Processing**: Decrypt and obfuscate data securely
6. **Tribute Minting**: Create NFT representing consumption

### Error Handling

- **Decryption Errors**: Invalid encryption parameters or corrupted data
- **Proof Verification**: ZK proof validation failures
- **Uniqueness Violations**: Duplicate tribute or CU hash submissions
- **Configuration Errors**: Invalid TEE or ZK setup parameters
- **Access Control**: Unauthorized configuration updates

### Best Practices

- **Key Rotation**: Regular rotation of TEE cryptographic keys
- **Secure Communication**: Always use encrypted channels for key exchange
- **Proof Validation**: Thorough ZK proof verification before tribute creation
- **Audit Logging**: Comprehensive logging of all tribute creation events
- **Error Recovery**: Graceful handling of decryption and validation failures

## Mathematical Models

### Blake3 Hash Functions

**Tribute Draft ID Generation**:
```
tribute_draft_id = Blake3(owner || worldwide_day)
```

**Tribute Hash for Merkle Tree**:
```
tribute_hash = Blake3(
    tribute_draft_id || settlement_currency || 
    settlement_base_amount || settlement_atto_amount ||
    nominal_base_qty || nominal_atto_qty || 
    concat(cu_hashes)
)
```

### HKDF Key Derivation

**Encryption Key Derivation**:
```
PRK = HKDF-Extract(salt, shared_secret)
encryption_key = HKDF-Expand(PRK, "tribute-factory-encryption", 32)
```

## Security Considerations

### Cryptographic Security

- **Forward Secrecy**: ECDHE provides forward secrecy for each session
- **Authenticated Encryption**: ChaCha20-Poly1305 prevents tampering
- **Key Isolation**: Each tribute uses unique ephemeral keys
- **Salt Randomness**: Cryptographically secure salt generation

### Data Privacy

- **L2-L1 Unlinkability**: TEE obfuscation breaks data correlation
- **Minimal Exposure**: Only necessary data exposed on L1
- **Zero-Knowledge Proofs**: Prove validity without revealing private data
- **Secure Deletion**: Ephemeral keys discarded after use

### Access Control

- **Owner Management**: Secure contract ownership controls
- **TEE Authorization**: Only authorized TEE can decrypt data
- **Tribute Uniqueness**: Cryptographic prevention of double-spending
- **Configuration Protection**: Secure updates to critical parameters

## License

This contract is part of the Q-Contracts ecosystem. See the main repository for licensing information.
