So, you have a nice `async fn` and you want to store a future it returns in a struct. There’s
no need for boxing or dynamic dispatch: you statically know the type. You just need to...

# name-it

```rust
#[name_it(Test)]
async fn add(x: i32, y: i32) -> i32 {
    do_something_very_async().await;
    x + y
}

let foo: Test = add(2, 3);
assert_eq!(block_on(foo), 5);
```

