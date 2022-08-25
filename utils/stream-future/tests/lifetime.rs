#![feature(generators)]

use stream_future::*;

#[tokio::test]
async fn lifetime() {
    #[stream("()", lifetime = "'a")]
    async fn foo<'a>(s: &'a str) {
        yield;
        println!("{}", s);
        yield;
    }

    foo("Hello world!").await;
}
