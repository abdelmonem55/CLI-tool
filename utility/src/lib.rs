///wrapper function above envsubst to handle default values in form of ${VAR:-default}
pub mod envsubst;
mod error;
mod openfaas;
pub use openfaas::*;

pub use error::*;

pub const HOST: &str = "localhost:8080";
pub const MODE_CHAR_DEVICE: u32 = 2097152;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        std::env::set_var("USER", "TEST");
        let res = crate::envsubst::substitute("${USER:-9} ${h:-gd}", &std::env::vars().collect())
            .unwrap();
        println!("res {}", res);
        assert_eq!(2 + 2, 4);
    }
}

pub fn is_default<T: Default + PartialEq>(t: &T) -> bool {
    *t == Default::default()
}
