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

assert_eq!(<MyGreeter as Greeter>::NAME, "Greeter");

let mut builder = planus::Builder::new();
let offset = HelloRequest::create(&mut builder, "Planus");
let data = builder.finish(offset, None);

let request = HelloRequestRef::read_as_root(data).unwrap();
let reply = MyGreeter.say_hello(request).unwrap();
assert_eq!(reply.message, "Hello, Planus!");

let reply = MyGreeter.say_hello_again(request).unwrap();
assert_eq!(reply.message, "Hello, Planus!");
