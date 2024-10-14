use thiserror::Error;

#[derive(Error, Debug)]
pub enum AMMError {
    #[error("Contract error")]
    ContractError(),
}

#[derive(Error, Debug)]
pub enum ArithmeticError {
    #[error("Division by zero error")]
    DivisionByZero(),
}

#[derive(Error, Debug)]
pub enum SwapSimulationError {
    #[error("Overflow Error")]
    Overflow(),
    #[error("Division by zero error")]
    DivisionByZero(),
}
