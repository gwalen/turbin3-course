use anchor_client::{Client, Cluster, Program};
use anchor_lang::{
    prelude::Pubkey,
    solana_program::{instruction::Instruction, system_program},
};
use anchor_spl::{
    associated_token::{self, get_associated_token_address_with_program_id},
    memo, token,
};
use escrow::{
    accounts, instruction,
    state::{escrow::Escrow, timed_escrow::TimedEscrow},
};
use solana_keypair::Keypair;
use std::rc::Rc;

pub struct EscrowClient {
    program: Program<Rc<Keypair>>,
    pub token_program_a: Pubkey,
    pub token_program_b: Pubkey,
}

impl EscrowClient {
    pub fn new() -> Self {
        let anchor_client =
            Client::new(Cluster::Localnet, Rc::new(Keypair::new_from_array([6; 32])));

        let program = anchor_client.program(escrow::ID).unwrap();
        Self {
            program,
            token_program_a: token::ID,
            token_program_b: token::ID,
        }
    }

    pub fn make(
        &self,
        maker: Pubkey,
        mint_a: Pubkey,
        mint_b: Pubkey,
        seed: u64,
        deposit: u64,
        receive: u64,
    ) -> Instruction {
        let (escrow, _) = Escrow::find_pda(&maker, seed);
        self.program
            .request()
            .accounts(accounts::Make {
                maker,
                mint_a,
                escrow,
                vault: get_associated_token_address_with_program_id(
                    &escrow,
                    &mint_a,
                    &self.token_program_a,
                ),
                maker_ata_a: get_associated_token_address_with_program_id(
                    &maker,
                    &mint_a,
                    &self.token_program_a,
                ),
                mint_b,
                associated_token_program: associated_token::ID,
                token_program_a: self.token_program_a,
                token_program_b: self.token_program_b,
                system_program: system_program::ID,
            })
            .args(instruction::Make {
                seed,
                deposit,
                receive,
            })
            .instructions()
            .remove(0)
    }

    pub fn take(
        &self,
        maker: Pubkey,
        taker: Pubkey,
        mint_a: Pubkey,
        mint_b: Pubkey,
        seed: u64,
    ) -> Instruction {
        let (escrow, _) = Escrow::find_pda(&maker, seed);
        self.program
            .request()
            .accounts(accounts::Take {
                maker,
                mint_a,
                escrow,
                vault: get_associated_token_address_with_program_id(
                    &escrow,
                    &mint_a,
                    &self.token_program_a,
                ),
                taker,
                mint_b,
                taker_ata_a: get_associated_token_address_with_program_id(
                    &taker,
                    &mint_a,
                    &self.token_program_a,
                ),
                taker_ata_b: get_associated_token_address_with_program_id(
                    &taker,
                    &mint_b,
                    &self.token_program_b,
                ),
                maker_ata_b: get_associated_token_address_with_program_id(
                    &maker,
                    &mint_b,
                    &self.token_program_b,
                ),
                associated_token_program: associated_token::ID,
                token_program_a: self.token_program_a,
                token_program_b: self.token_program_b,
                memo_program: memo::ID,
                system_program: system_program::ID,
            })
            .args(instruction::Take {})
            .instructions()
            .remove(0)
    }

    pub fn refund(&self, maker: Pubkey, mint_a: Pubkey, seed: u64) -> Instruction {
        let (escrow, _) = Escrow::find_pda(&maker, seed);
        self.program
            .request()
            .accounts(accounts::Refund {
                maker,
                mint_a,
                escrow,
                vault: get_associated_token_address_with_program_id(
                    &escrow,
                    &mint_a,
                    &self.token_program_a,
                ),
                maker_ata_a: get_associated_token_address_with_program_id(
                    &maker,
                    &mint_a,
                    &self.token_program_a,
                ),
                associated_token_program: associated_token::ID,
                token_program: self.token_program_a,
                system_program: system_program::ID,
            })
            .args(instruction::Refund {})
            .instructions()
            .remove(0)
    }
    pub fn make_timed(
        &self,
        maker: Pubkey,
        mint_a: Pubkey,
        mint_b: Pubkey,
        seed: u64,
        deposit: u64,
        receive: u64,
        expires_at: i64,
    ) -> Instruction {
        let (escrow, _) = TimedEscrow::find_pda(&maker, seed);
        self.program
            .request()
            .accounts(accounts::MakeTimed {
                maker,
                mint_a,
                escrow,
                vault: get_associated_token_address_with_program_id(
                    &escrow,
                    &mint_a,
                    &self.token_program_a,
                ),
                maker_ata_a: get_associated_token_address_with_program_id(
                    &maker,
                    &mint_a,
                    &self.token_program_a,
                ),
                mint_b,
                associated_token_program: associated_token::ID,
                token_program_a: self.token_program_a,
                token_program_b: self.token_program_b,
                system_program: system_program::ID,
            })
            .args(instruction::MakeTimed {
                seed,
                deposit,
                receive,
                expires_at,
            })
            .instructions()
            .remove(0)
    }

    pub fn take_expired(
        &self,
        maker: Pubkey,
        taker: Pubkey,
        mint_a: Pubkey,
        mint_b: Pubkey,
        seed: u64,
    ) -> Instruction {
        let (escrow, _) = TimedEscrow::find_pda(&maker, seed);
        self.program
            .request()
            .accounts(accounts::TakeExpired {
                maker,
                mint_a,
                escrow,
                vault: get_associated_token_address_with_program_id(
                    &escrow,
                    &mint_a,
                    &self.token_program_a,
                ),
                taker,
                mint_b,
                taker_ata_a: get_associated_token_address_with_program_id(
                    &taker,
                    &mint_a,
                    &self.token_program_a,
                ),
                taker_ata_b: get_associated_token_address_with_program_id(
                    &taker,
                    &mint_b,
                    &self.token_program_b,
                ),
                maker_ata_b: get_associated_token_address_with_program_id(
                    &maker,
                    &mint_b,
                    &self.token_program_b,
                ),
                associated_token_program: associated_token::ID,
                token_program_a: self.token_program_a,
                token_program_b: self.token_program_b,
                memo_program: memo::ID,
                system_program: system_program::ID,
            })
            .args(instruction::TakeExpired {})
            .instructions()
            .remove(0)
    }

    pub fn refund_expired(&self, maker: Pubkey, mint_a: Pubkey, seed: u64) -> Instruction {
        let (escrow, _) = TimedEscrow::find_pda(&maker, seed);
        self.program
            .request()
            .accounts(accounts::RefundExpired {
                maker,
                mint_a,
                escrow,
                vault: get_associated_token_address_with_program_id(
                    &escrow,
                    &mint_a,
                    &self.token_program_a,
                ),
                maker_ata_a: get_associated_token_address_with_program_id(
                    &maker,
                    &mint_a,
                    &self.token_program_a,
                ),
                associated_token_program: associated_token::ID,
                token_program: self.token_program_a,
                system_program: system_program::ID,
            })
            .args(instruction::RefundExpired {})
            .instructions()
            .remove(0)
    }
}
