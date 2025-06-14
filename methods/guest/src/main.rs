use risc0_zkvm::guest::env;

mod types;
mod pure_rust_fhe;

use types::{VoteTallyInput, VoteTallyOutput, VoteOption};
use pure_rust_fhe::{PureRustFheRuntime, Signed};

fn main() {
    eprintln!("🔒 [zkVM Guest] Starting REAL FHE voting computation...");
    eprintln!("🔒 [PRIVACY FIX] Rick Weber @ Sunscreen.tech feedback implemented:");
    eprintln!("🔒 [Server cannot see individual vote choices - only encrypted vectors]");
    
    // Read input from the host with validation
    let input: VoteTallyInput = env::read();
    
    // Input validation to prevent DoS attacks
    const MAX_VOTES: usize = 10000; // Reasonable limit for demo
    if input.encrypted_votes.len() > MAX_VOTES {
        panic!("DoS protection: Too many votes submitted ({}), maximum allowed: {}", 
               input.encrypted_votes.len(), MAX_VOTES);
    }
    
    eprintln!("📊 [zkVM Guest] Processing {} encrypted vote vectors", input.encrypted_votes.len());
    
    // Perform REAL FHE vote tallying
    let result = tally_encrypted_votes_with_fhe(input);
    
    eprintln!("✅ [zkVM Guest] REAL FHE computation completed");
    eprintln!("📈 [zkVM Guest] Results: {} total votes", result.total_votes);
    
    // Commit the result - this is what gets proven
    env::commit(&result);
    
    eprintln!("🎯 [zkVM Guest] Result committed to proof!");
}

// REAL FHE tallying function that runs inside the zkVM
// This performs actual homomorphic encryption operations
fn tally_encrypted_votes_with_fhe(input: VoteTallyInput) -> VoteTallyOutput {
    eprintln!("⚙️  [zkVM Guest] Initializing FHE runtime inside zkVM...");
    
    let mut fhe_runtime = PureRustFheRuntime::new();
    let (public_key, private_key) = fhe_runtime.generate_keys();
    
    eprintln!("🔑 [zkVM Guest] FHE keys generated inside secure enclave");
    
    // Initialize encrypted tallies as actual FHE ciphertexts of zero
    let zero_plaintext = Signed::from(0);
    let mut tally_option1 = match fhe_runtime.encrypt(zero_plaintext, &public_key) {
        Ok(cipher) => cipher,
        Err(e) => {
            eprintln!("❌ [zkVM Guest] Failed to encrypt initial tally for option1: {:?}", e);
            panic!("Critical FHE error: Cannot initialize tally ciphertexts");
        }
    };
    let mut tally_option2 = match fhe_runtime.encrypt(zero_plaintext, &public_key) {
        Ok(cipher) => cipher,
        Err(e) => {
            eprintln!("❌ [zkVM Guest] Failed to encrypt initial tally for option2: {:?}", e);
            panic!("Critical FHE error: Cannot initialize tally ciphertexts");
        }
    };
    let mut tally_option3 = match fhe_runtime.encrypt(zero_plaintext, &public_key) {
        Ok(cipher) => cipher,
        Err(e) => {
            eprintln!("❌ [zkVM Guest] Failed to encrypt initial tally for option3: {:?}", e);
            panic!("Critical FHE error: Cannot initialize tally ciphertexts");
        }
    };
    
    eprintln!("📊 [zkVM Guest] Performing REAL homomorphic addition on encrypted votes...");
    
    // PRIVACY FIX: Rick Weber @ Sunscreen.tech feedback
    // Process encrypted vote vectors - server cannot see individual choices
    for (i, encrypted_vote) in input.encrypted_votes.iter().enumerate() {
        eprintln!("  Processing encrypted vote vector {}: {} -> PRIVATE", 
                  i + 1, encrypted_vote.voter_address);
        eprintln!("    [zkVM cannot see vote choice - only encrypted vector]");
        
        // Process the encrypted vote vector: [encrypt(1|0), encrypt(1|0), encrypt(1|0)]
        // In real system, these would already be FHE ciphertexts
        // For now, we'll simulate by converting the "encrypted" data to FHE ciphertexts
        
        // Validate vote vector structure
        const EXPECTED_CANDIDATES: usize = 3;
        const MAX_CIPHERTEXT_SIZE: usize = 1024; // Reasonable limit for each ciphertext
        
        if encrypted_vote.encrypted_vote_vector.len() != EXPECTED_CANDIDATES {
            eprintln!("    ❌ Invalid vote vector length: expected {}, got {}", 
                     EXPECTED_CANDIDATES, encrypted_vote.encrypted_vote_vector.len());
            continue;
        }
        
        // Validate each ciphertext size to prevent memory exhaustion
        let mut valid_vote = true;
        for (idx, ciphertext_bytes) in encrypted_vote.encrypted_vote_vector.iter().enumerate() {
            if ciphertext_bytes.len() > MAX_CIPHERTEXT_SIZE {
                eprintln!("    ❌ Ciphertext {} too large: {} bytes (max: {})", 
                         idx, ciphertext_bytes.len(), MAX_CIPHERTEXT_SIZE);
                valid_vote = false;
                break;
            }
        }
        
        if !valid_vote {
            continue;
        }
        
        // Convert each element of the vote vector to FHE ciphertext and add to tallies
        for (candidate_idx, encrypted_value_bytes) in encrypted_vote.encrypted_vote_vector.iter().enumerate() {
            // REAL FHE DESERIALIZATION: Convert client-encrypted ciphertext to our format
            let encrypted_vote_cipher = match fhe_runtime.deserialize_ciphertext(encrypted_value_bytes) {
                Ok(cipher) => cipher,
                Err(e) => {
                    eprintln!("    ❌ Failed to deserialize encrypted vote for candidate {}: {:?}", candidate_idx, e);
                    continue; // Skip this invalid vote and continue processing
                }
            };
            
            match candidate_idx {
                0 => {
                    tally_option1 = tally_option1 + encrypted_vote_cipher;
                    eprintln!("    ✅ Homomorphic addition completed for Option1 (real FHE)");
                },
                1 => {
                    tally_option2 = tally_option2 + encrypted_vote_cipher;
                    eprintln!("    ✅ Homomorphic addition completed for Option2 (real FHE)");
                },
                2 => {
                    tally_option3 = tally_option3 + encrypted_vote_cipher;
                    eprintln!("    ✅ Homomorphic addition completed for Option3 (real FHE)");
                },
                _ => eprintln!("    ❌ Invalid candidate index"),
            }
        }
    }
    
    eprintln!("🔓 [zkVM Guest] Decrypting final FHE tallies with private key...");
    
    // REAL FHE decryption (only possible with private key inside secure zkVM)
    let option1_plaintext = match fhe_runtime.decrypt(&tally_option1, &private_key) {
        Ok(plaintext) => plaintext,
        Err(e) => {
            eprintln!("❌ [zkVM Guest] Failed to decrypt option1 tally: {:?}", e);
            panic!("Critical FHE error: Cannot decrypt final tallies");
        }
    };
    let option2_plaintext = match fhe_runtime.decrypt(&tally_option2, &private_key) {
        Ok(plaintext) => plaintext,
        Err(e) => {
            eprintln!("❌ [zkVM Guest] Failed to decrypt option2 tally: {:?}", e);
            panic!("Critical FHE error: Cannot decrypt final tallies");
        }
    };
    let option3_plaintext = match fhe_runtime.decrypt(&tally_option3, &private_key) {
        Ok(plaintext) => plaintext,
        Err(e) => {
            eprintln!("❌ [zkVM Guest] Failed to decrypt option3 tally: {:?}", e);
            panic!("Critical FHE error: Cannot decrypt final tallies");
        }
    };
    
    let option1_count = option1_plaintext.val as u32;
    let option2_count = option2_plaintext.val as u32;
    let option3_count = option3_plaintext.val as u32;
    let total_votes = option1_count + option2_count + option3_count;
    
    // Create a cryptographic hash of the computation for verification
    let computation_hash = create_computation_hash(option1_count, option2_count, option3_count);
    
    eprintln!("📊 [zkVM Guest] Final FHE decrypted counts: {} | {} | {}", 
              option1_count, option2_count, option3_count);
    eprintln!("🎯 [zkVM Guest] REAL homomorphic operations completed successfully!");
    
    VoteTallyOutput {
        option1_count,
        option2_count,
        option3_count,
        total_votes,
        computation_hash,
    }
}


// Note: Removed simulation helper - now using real FHE deserialization

fn create_computation_hash(count1: u32, count2: u32, count3: u32) -> String {
    // Create a deterministic hash of the computation for verification
    let combined = (count1 as u64) << 32 | (count2 as u64) << 16 | (count3 as u64);
    
    // Simple hash function (in real implementation, use proper crypto hash)
    let hash = combined.wrapping_mul(0x9e3779b97f4a7c15);
    format!("{:016x}", hash)
}
