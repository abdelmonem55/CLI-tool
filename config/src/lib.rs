pub mod config_file;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let expand =
            utility::envsubst::substitute("~/${TEST:-2}/${HOME:-ds}", &std::env::vars().collect());
        println!("{:?}", expand);
        assert_eq!(2 + 2, 4);
    }
}
