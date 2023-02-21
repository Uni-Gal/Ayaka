fn main() {
    #[cfg(feature = "parser")]
    lalrpop::process_root().unwrap();
}
