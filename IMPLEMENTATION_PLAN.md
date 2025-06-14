# Single Election Key Implementation Plan

**Decision**: Implement Option A (Single Election Key Architecture)  
**Consensus**: Both O3 and Gemini strongly recommend this approach  

## Expert Review Consensus

### O3 (Cryptographic Authority)
- ✅ **Only viable option** given zkVM resource constraints
- ✅ **Industry standard** (Helios-FHE, Inferno, CH vote prototype)
- ✅ **Mathematically sound** with immediate correctness
- ⚠️ Requires **hardened threshold key ceremony**

### Gemini (Engineering Perspective)  
- ✅ **Practical and efficient** for immediate deployment
- ✅ **Manageable security trade-offs** with proper protocols
- ⚠️ **Operational complexity** in key management
- ⚠️ **Edge cases** requiring careful attention

---

## Implementation Strategy

### Phase 1: Core Architecture Fix (Sprint 1)
**Goal**: Resolve mathematical inconsistency immediately

#### 1.1 Shared Election Key Generation
```rust
// New: Election authority generates shared key pair
pub struct ElectionAuthority {
    // For demo: single authority, production: threshold DKG
}

impl ElectionAuthority {
    pub fn generate_election_keys() -> (PublicKey, PrivateKey) {
        // Cryptographically secure key generation
        // Same parameters as current implementation
    }
    
    pub fn get_public_key_commitment() -> String {
        // H(pk_election || params) for verification
    }
}
```

#### 1.2 Client Modification
```rust
// Modified: Clients use shared election public key
impl FheClient {
    pub fn new(election_public_key: &PublicKey) -> Self {
        FheClient {
            runtime: PureRustFheRuntime::new(),
            election_public_key: election_public_key.clone(), // Shared key
        }
    }
    
    pub fn encrypt_vote_vector(&self, vote_choice: VoteOption) -> Result<Vec<Vec<u8>>, FheClientError> {
        // All votes encrypted under shared election_public_key
        let ciphertext = self.runtime.encrypt(plaintext, &self.election_public_key)?;
    }
}
```

#### 1.3 Guest Modification
```rust
// Modified: Guest uses pre-committed election private key
fn tally_encrypted_votes_with_fhe(input: VoteTallyInput) -> VoteTallyOutput {
    // Remove: let (public_key, private_key) = fhe_runtime.generate_keys();
    
    // Add: Load pre-committed election keys
    let election_keys = load_election_keys(); // From secure storage
    let (public_key, private_key) = election_keys;
    
    // Rest of implementation remains the same
    // All ciphertexts now mathematically compatible
}
```

### Phase 2: Security Hardening (Sprint 2)
**Goal**: Implement production-level security measures

#### 2.1 Secret Memory Protection
```rust
// Implement zkVM secret memory protection
use risc0_zkvm::secret_mem;

fn tally_encrypted_votes_with_fhe(input: VoteTallyInput) -> VoteTallyOutput {
    // Protect private key from execution trace
    let private_key = secret_mem::protect(|| {
        load_election_private_key()
    });
    
    // Use protected key for decryption only
    let final_tallies = secret_mem::with_protected(&private_key, |sk| {
        fhe_runtime.decrypt(&encrypted_tallies, sk)
    });
    
    // Automatic zeroization when out of scope
}
```

#### 2.2 Key Commitment System
```rust
// Add key commitment and verification
pub struct ElectionSetup {
    pub election_id: String,
    pub public_key_hash: String,
    pub parameters_hash: String,
    pub setup_timestamp: u64,
}

impl ElectionSetup {
    pub fn commit_to_blockchain(&self) -> Result<TxHash, Error> {
        // Publish H(pk_election || params) on-chain before voting
    }
    
    pub fn verify_commitment(&self, public_key: &PublicKey) -> bool {
        // Verify public key matches committed hash
    }
}
```

#### 2.3 Input Validation Enhancement
```rust
// Enhanced validation for election context
fn validate_election_context(input: &VoteTallyInput) -> Result<(), ValidationError> {
    // Verify all votes reference correct election_id
    // Verify vote timestamps within election window
    // Verify no duplicate voter addresses
    // Enhanced DoS protection
}
```

### Phase 3: Threshold Security (Sprint 3-4)
**Goal**: Remove single point of failure in key management

#### 3.1 Threshold Key Generation (Future)
```rust
// Placeholder for threshold DKG implementation
pub struct ThresholdElectionAuthority {
    trustees: Vec<TrusteeId>,
    threshold: usize, // t-of-n
}

impl ThresholdElectionAuthority {
    pub fn distributed_key_generation(
        trustees: &[TrusteeId], 
        threshold: usize
    ) -> Result<(PublicKey, Vec<SecretShare>), DkgError> {
        // Implementation of verifiable DKG
        // Each trustee gets secret share
        // Public key deterministically derived
    }
}
```

#### 3.2 Secret Sharing Integration
```rust
// zkVM secret reconstruction
fn reconstruct_election_key(shares: &[SecretShare]) -> PrivateKey {
    // Combine threshold shares inside secure enclave
    // Immediate zeroization after use
}
```

---

## File Changes Required

### Phase 1 Changes (Immediate)

#### `host/src/fhe_client.rs`
- **Modify constructor** to accept election public key parameter
- **Remove individual key generation**
- **Update all encryption** to use shared election key

#### `methods/guest/src/main.rs`  
- **Remove guest key generation**
- **Add election key loading** mechanism
- **Implement secret memory protection**

#### `host/src/main.rs`
- **Add election authority** initialization
- **Generate shared election keys** before vote creation
- **Pass election public key** to all clients

#### New Files
- `host/src/election_authority.rs` - Election key management
- `host/src/key_commitment.rs` - Blockchain commitment system
- `shared/election_types.rs` - Common election data structures

---

## Testing Strategy

### Unit Tests
- ✅ **Key generation** produces valid FHE key pairs
- ✅ **Client encryption** under shared key works correctly
- ✅ **Guest decryption** with election private key succeeds
- ✅ **Homomorphic operations** maintain correctness

### Integration Tests  
- ✅ **End-to-end voting** with shared keys
- ✅ **Multiple clients** using same election key
- ✅ **Proof generation** and verification
- ✅ **Error handling** for malformed inputs

### Security Tests
- ✅ **Secret memory protection** prevents key leakage
- ✅ **Key commitment verification** works correctly
- ✅ **Input validation** blocks malicious inputs
- ✅ **Side-channel resistance** validation

---

## Success Criteria

### Functional Requirements
1. ✅ **Mathematical correctness**: All ciphertexts under same key
2. ✅ **Performance**: Proof generation <60 seconds  
3. ✅ **Security**: Private key protected in zkVM
4. ✅ **Verification**: Key commitments publicly verifiable

### Security Requirements
1. ✅ **No key leakage** via execution trace
2. ✅ **Proper error handling** for all failure modes
3. ✅ **Input validation** prevents DoS attacks
4. ✅ **Cryptographic parameters** meet 128-bit security

### Quality Requirements
1. ✅ **Code coverage** >90% for critical paths
2. ✅ **Documentation** for all public APIs
3. ✅ **Performance benchmarks** established
4. ✅ **Security audit** readiness

---

## Risk Mitigation

### Technical Risks
- **Implementation complexity**: Incremental development with testing
- **Performance regression**: Continuous benchmarking
- **Security vulnerabilities**: Expert review at each phase

### Operational Risks  
- **Key management**: Start with single authority, evolve to threshold
- **Disaster recovery**: Document key recovery procedures
- **User experience**: Maintain backward compatibility

---

## Next Steps

1. **Implement Phase 1** (Core Architecture Fix)
2. **Get code review** from Gemini after implementation
3. **Security testing** and validation
4. **Performance benchmarking**
5. **Get final O3 review** before merge

This plan addresses both O3's cryptographic requirements and Gemini's engineering concerns while providing a clear path to production-ready implementation.