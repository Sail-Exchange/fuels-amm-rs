use fuels::types::errors::Error;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AMMError {
    #[error("Contract error")]
    ContractError,
    #[error("Simulation Error")]
    SimulationError(#[from] Error),
}

#[derive(Error, Debug)]
pub enum ArithmeticError {
    #[error("Division by zero error")]
    DivisionByZero,
    #[error("Rounding Error")]
    RoundingError,
    #[error("Y is zero")]
    YIsZero,
}

#[derive(Error, Debug)]
pub enum SwapSimulationError {
    #[error("Overflow Error")]
    Overflow,
    #[error("Division by zero error")]
    DivisionByZero,
}
