// #![no_std]

#[cfg(not(feature = "no-entrypoint"))]
mod entrypoint;

#[cfg(feature = "std")]
extern crate std;

mod error;
mod instruction;
mod state;

pinocchio_pubkey::declare_id!("7KuDrDJsLa2iKcUovWs7DFNYRdYJ12MyKyaJwnqmhSxy");