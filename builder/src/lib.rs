pub mod build;
pub mod copy;
pub mod publish;

pub use copy::*;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
