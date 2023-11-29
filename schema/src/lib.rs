pub mod describe;
pub mod image;
pub mod knative;
pub mod metadata;
pub mod openfaas;
pub mod secret;
pub mod store;
pub mod store_item;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
