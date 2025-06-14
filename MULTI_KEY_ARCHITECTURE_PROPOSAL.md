# Multi-Key Architecture Decision Document

**Issue**: Critical architectural inconsistency in FHE key management identified by expert reviews.

## Current Problem

### Client Implementation (`host/src/fhe_client.rs:68`)
```rust
// Each client creates individual FHE keys
let mut runtime = PureRustFheRuntime::new();
let (public_key, _private_key) = runtime.generate_keys(); // Unique per client
```

### Guest Implementation (`methods/guest/src/main.rs:44`)
```rust
// Guest generates its own keys, expecting all votes under same key
let (public_key, private_key) = fhe_runtime.generate_keys();
```

### Mathematical Issue
- **BFV/RLWE Limitation**: Cannot add `Enc_K1(m1) + Enc_K2(m2)` where K1 ≠ K2
- **Current Result**: Mathematically undefined operation, incorrect tallies
- **Security Impact**: Undermines entire cryptographic foundation

## Solution Options

### Option A: Single Election Key Architecture (Recommended)

#### Design
```rust
// 1. Election authority generates key pair before voting
let (election_pk, election_sk) = generate_election_keys();

// 2. Clients encrypt under shared election public key
let encrypted_vote = encrypt_under_shared_key(vote, &election_pk);

// 3. Guest uses pre-committed election private key (sealed in zkVM)
let election_sk = risc0_zkvm::secret_mem::load_election_key();
let tally = decrypt_final_results(&encrypted_tallies, &election_sk);
```

#### Advantages
- ✅ **Mathematically sound**: All ciphertexts under same key
- ✅ **Simple implementation**: Minimal code changes required
- ✅ **Performance**: No key-switching overhead
- ✅ **Security**: Election key sealed in zkVM, only accessible for tallying
- ✅ **Verifiable**: Key commitment can be proven in advance

#### Trade-offs
- ⚠️ **Key management**: Requires trusted election authority for key generation
- ⚠️ **Key ceremony**: Need secure setup phase before voting

### Option B: True Multi-Key FHE

#### Design
```rust
// 1. Each voter generates their own key pair
let (voter_pk_i, voter_sk_i) = voter_i.generate_keys();

// 2. Implement key-switching for homomorphic operations
let switched_cipher = key_switch(cipher_under_pk_i, &eval_key_i_to_common);
let tally = add_switched_ciphertexts(&switched_ciphers);

// 3. Use evaluation keys for cross-key operations
let eval_keys = generate_evaluation_keys(&voter_keys, &common_key);
```

#### Advantages
- ✅ **No shared secrets**: Each voter controls their own keys
- ✅ **Decentralized**: No trusted election authority needed
- ✅ **Individual control**: Voters can prove their vote was included

#### Trade-offs
- ❌ **Complex implementation**: Requires evaluation keys, key-switching
- ❌ **Performance overhead**: Key-switching operations are expensive
- ❌ **Parameter constraints**: May not fit in current zkVM memory limits
- ❌ **Development time**: Significantly more complex to implement correctly

## Security Analysis

### Option A Security Properties
- **Vote Privacy**: Individual votes encrypted, only tallies revealed
- **Result Integrity**: zkVM proof ensures correct homomorphic operations
- **Key Security**: Election private key sealed in zkVM execution
- **Verifiability**: Public key commitment allows verification of setup

### Option B Security Properties  
- **Enhanced Vote Privacy**: No shared secrets between voters
- **Individual Verifiability**: Each voter can verify their vote inclusion
- **Distributed Trust**: No single point of failure in key management
- **Complex Attack Surface**: More cryptographic components = more potential vulnerabilities

## Implementation Complexity

### Option A: Estimated 1-2 sprints
1. Modify client to accept shared public key
2. Implement election key sealing in guest
3. Add key commitment verification
4. Update tests and documentation

### Option B: Estimated 4-6 sprints
1. Implement evaluation key generation
2. Add key-switching algorithms
3. Modify homomorphic operations for multi-key
4. Extensive testing of cryptographic correctness
5. Memory optimization for zkVM constraints

## Recommendation

**Choose Option A: Single Election Key Architecture**

### Rationale
1. **Immediate Fix**: Resolves critical mathematical inconsistency quickly
2. **Solid Foundation**: Provides correct cryptographic base for future development
3. **Expert Validation**: Aligns with O3's suggestion of "single election key"
4. **Practical Constraints**: Works within current zkVM memory and performance limits
5. **Industry Standard**: Many production FHE voting systems use shared election keys

### Implementation Plan
1. **Phase 1**: Implement shared key generation and distribution
2. **Phase 2**: Seal election private key in zkVM guest
3. **Phase 3**: Add key commitment and verification mechanisms
4. **Phase 4**: Comprehensive testing and security validation

## Questions for O3 Review

1. **Architecture Choice**: Do you agree Option A is the right approach for our current constraints?

2. **Key Management**: What's the best practice for secure election key generation and distribution?

3. **zkVM Integration**: Any concerns with sealing the election private key in the guest program?

4. **Security Trade-offs**: Are we making reasonable security vs. implementation complexity trade-offs?

5. **Future Migration**: If we implement Option A now, how difficult would migration to Option B be later?

Please provide architectural guidance on these decisions and any additional security considerations we should address.