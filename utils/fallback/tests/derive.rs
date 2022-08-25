use fallback::*;

#[derive(FallbackSpec)]
struct Foo {
    data1: i32,
    data2: String,
}

#[test]
fn derive() {
    let data = Foo {
        data1: 123,
        data2: "Hello".to_string(),
    };

    let data = Fallback::new(None, Some(data));
    let data = data.spec();

    assert_eq!(data.data1.unzip(), (None, Some(123)));
    assert_eq!(data.data2.unzip(), (None, Some("Hello".to_string())));
}
