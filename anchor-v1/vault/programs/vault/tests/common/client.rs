use {
    anchor_client::{Client, Cluster, Program},
    anchor_lang::{
        prelude::Pubkey,
        solana_program::{instruction::Instruction, system_program},
    },
    solana_keypair::Keypair,
    std::rc::Rc,
    vault::{accounts, instruction, state::vault_state::VaultState},
};

pub struct VaultClient {
    program: Program<Rc<Keypair>>,
}

impl VaultClient {
    pub fn new() -> Self {
        let client = Client::new(Cluster::Localnet, Rc::new(Keypair::new()));
        let program = client.program(vault::ID).unwrap();
        Self { program }
    }

    pub fn initialize(&self, user: Pubkey) -> Instruction {
        let (vault_state, _) = VaultState::find_pda(&user);
        let (vault, _) = VaultState::find_vault_pda(&vault_state);
        self.program
            .request()
            .accounts(accounts::Initialize {
                user,
                vault_state,
                vault,
                system_program: system_program::ID,
            })
            .args(instruction::Initialize {})
            .instructions()
            .remove(0)
    }

    pub fn deposit(&self, user: Pubkey, amount: u64) -> Instruction {
        let (vault_state, _) = VaultState::find_pda(&user);
        let (vault, _) = VaultState::find_vault_pda(&vault_state);
        self.program
            .request()
            .accounts(accounts::Deposit {
                user,
                vault,
                vault_state,
                system_program: system_program::ID,
            })
            .args(instruction::Deposit { amount })
            .instructions()
            .remove(0)
    }

    pub fn withdraw(&self, user: Pubkey, amount: u64) -> Instruction {
        let (vault_state, _) = VaultState::find_pda(&user);
        let (vault, _) = VaultState::find_vault_pda(&vault_state);
        self.program
            .request()
            .accounts(accounts::Withdraw {
                user,
                vault,
                vault_state,
                system_program: system_program::ID,
            })
            .args(instruction::Withdraw { amount })
            .instructions()
            .remove(0)
    }

    pub fn close(&self, user: Pubkey) -> Instruction {
        let (vault_state, _) = VaultState::find_pda(&user);
        let (vault, _) = VaultState::find_vault_pda(&vault_state);
        self.program
            .request()
            .accounts(accounts::Close {
                user,
                vault,
                vault_state,
                system_program: system_program::ID,
            })
            .args(instruction::Close {})
            .instructions()
            .remove(0)
    }
}
