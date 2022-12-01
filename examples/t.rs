use std::future::Future;

use name_it::name_it;

#[derive(Debug)]
struct Message;

trait Dumper {
    type DumperFut<'a>: Future<Output = ()>;

    fn dump(&self, message: Message) -> DumperFut<'_>;
}

struct StdoutDumper;

#[name_it(DumperFut)]
async fn dump_impl(_dumper: &StdoutDumper, message: Message) {
    println!("dump: {message:?}");
}

impl Dumper for StdoutDumper {
    type DumperFut<'a> = DumperFut<'a>;

    fn dump(&self, message: Message) -> DumperFut<'_> {
        dump_impl(self, message)
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    StdoutDumper.dump(Message).await;
}
