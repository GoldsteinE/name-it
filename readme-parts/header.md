So, you have a nice `async fn` and you want to store a future it returns in a struct. There’s
no need for boxing or dynamic dispatch: you statically know the type. You just need to...

# name-it

