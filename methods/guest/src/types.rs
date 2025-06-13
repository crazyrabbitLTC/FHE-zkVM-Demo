use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum VoteOption {
    Option1 = 1,
    Option2 = 2,
    Option3 = 3,
}

impl VoteOption {
    pub fn description(&self) -> &'static str {
        match self {
            VoteOption::Option1 => "Increase block size",
            VoteOption::Option2 => "Implement Layer 2 scaling", 
            VoteOption::Option3 => "Maintain current parameters",
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct VoteTallyInput {
    pub encrypted_votes: Vec<EncryptedVote>,
}

#[derive(Serialize, Deserialize)]
pub struct EncryptedVote {
    pub voter_address: String,
    // PRIVACY FIX: Rick Weber @ Sunscreen.tech feedback
    // Instead of revealing vote choice, encrypt full vote vector
    pub encrypted_vote_vector: Vec<Vec<u8>>, // [encrypt(1|0), encrypt(1|0), encrypt(1|0)] for each candidate
    pub signature: String, // Voter signature for authentication
    
    // Keep for demo purposes to verify correctness, but this would be removed in production
    pub actual_choice: VoteOption, // Only for verification - NOT sent in real system
}

#[derive(Serialize, Deserialize)]
pub struct VoteTallyOutput {
    pub option1_count: u32,
    pub option2_count: u32, 
    pub option3_count: u32,
    pub total_votes: u32,
    pub computation_hash: String, // Hash of the computation for verification
}