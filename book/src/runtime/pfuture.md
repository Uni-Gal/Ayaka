# Progress future
`ProgressFuture` is a future which also implements `Stream`.
The items iterated from `Stream` represent working states.

``` rust,ignore
// The iterated progress.
#[derive(Debug)]
enum Prog {
    Stage1,
    Stage2,
    End,
}

// The work function.
// You should specify the future type and progress type.
fn foo() -> ProgressFuture<impl Future<Output = Result<i32>>, Prog> {
    ProgressFuture::new(async move |tx| {
        tx.send(Prog::Stage1)?;
        // some works...
        tx.send(Prog::Stage2)?;
        // some other works...
        tx.send(Prog::End)?;
        Ok(0)
    })
}

let bar = foo();
// The object should be pinned, required by `StreamExt`.
tokio::pin!(bar);
// Iterate the progress.
while let Some(prog) = bar.next().await {
    println!("{:?}", prog);
}
// Await the future.
let bar = bar.await?;
assert_eq!(bar, 0);
```

If you don't want the progress, await the future directly.
