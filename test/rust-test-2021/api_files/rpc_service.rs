use core::{
    future::Future,
    pin::pin,
    task::{Context, Poll, Waker},
};

use planus::ReadAsRoot;

struct MyGreeter;

impl Greeter for MyGreeter {
    type Error = planus::Error;

    fn say_hello(&self, request: HelloRequestRef<'_>) -> Result<HelloReply, Self::Error> {
        Ok(HelloReply {
            message: format!("Hello, {}!", request.name()?.unwrap_or("world")),
        })
    }

    fn say_hello_again(&self, request: HelloRequestRef<'_>) -> Result<HelloReply, Self::Error> {
        self.say_hello(request)
    }
}

struct MyAsyncGreeter;

impl GreeterAsync for MyAsyncGreeter {
    type Error = planus::Error;

    async fn say_hello(&self, request: HelloRequestRef<'_>) -> Result<HelloReply, Self::Error> {
        Ok(HelloReply {
            message: format!("Hello, {}!", request.name()?.unwrap_or("world")),
        })
    }

    async fn say_hello_again(
        &self,
        request: HelloRequestRef<'_>,
    ) -> Result<HelloReply, Self::Error> {
        self.say_hello(request).await
    }
}

assert_eq!(<MyGreeter as Greeter>::NAME, "Greeter");
assert_eq!(<MyAsyncGreeter as GreeterAsync>::NAME, "Greeter");

let mut builder = planus::Builder::new();
let offset = HelloRequest::create(&mut builder, "Planus");
let data = builder.finish(offset, None);

let request = HelloRequestRef::read_as_root(data).unwrap();
let reply = MyGreeter.say_hello(request).unwrap();
assert_eq!(reply.message, "Hello, Planus!");

let reply = MyGreeter.say_hello_again(request).unwrap();
assert_eq!(reply.message, "Hello, Planus!");

// The futures of `GreeterAsync` have no await points, so they complete on
// the first poll and can be driven without an executor.
let mut cx = Context::from_waker(Waker::noop());
let mut future = pin!(MyAsyncGreeter.say_hello_again(request));
let Poll::Ready(reply) = future.as_mut().poll(&mut cx) else {
    panic!("future should be ready on the first poll");
};
assert_eq!(reply.unwrap().message, "Hello, Planus!");
