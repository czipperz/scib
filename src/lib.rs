pub mod types;
pub use types::*;
pub mod instance;
pub use instance::*;
mod lex;
mod parse;
mod eval;
mod builtins;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
    }
}
