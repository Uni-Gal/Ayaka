#![feature(generators)]

use progress_future::*;

#[tokio::test]
async fn lifetime() {
    #[progress("()", lifetime = "'a")]
    async fn foo<'a>(s: &'a str) {
        yield;
        println!("{}", s);
        yield;
    }

    foo("Hello world!").await;
}
