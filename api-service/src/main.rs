// This Rest(Rocket) API will have 2 entry points for the first iteration of the demo:
/*

    Transfer -> inserts a transfer into the local mock storage

    Submit -> produces a proof for current batch and submits it to the L1
    How is a proof produced?
        1. the local storage is read (current tree, current mock state which includes balances and transactions)
        2. the prove_state_transition function in 'host' is called with the local state as input
        3. the proof struct is returned that can be submitted to the L1 using the L1 client.
        The contract being called is the Tree/State contract, not the deposit contract!
        The Tree/State contract utilizes the on-chain verifier / host function
*/
use std::convert::Infallible;
use std::net::SocketAddr;
use http_body_util::Full;
use hyper::body::Bytes;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response};
use hyper_util::rt::TokioIo;
use tokio::net::TcpListener;

async fn hello(_: Request<hyper::body::Incoming>) -> Result<Response<Full<Bytes>>, Infallible> {
    Ok(Response::new(Full::new(Bytes::from("Hello, World!"))))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let host: [u8;4] = [127,0,0,1]; 
    let port: u16 = 3000;
    let addr = SocketAddr::from((host, port));

    // We create a TcpListener and bind it to 127.0.0.1:3000
    let listener = TcpListener::bind(addr).await?;

    // We start a loop to continuously accept incoming connections
    loop {
        println!("[Kairos API server] @ {:?}:{}", &host, &port);
        let (stream, _) = listener.accept().await?;

        // Use an adapter to access something implementing `tokio::io` traits as if they implement
        // `hyper::rt` IO traits.
        let io = TokioIo::new(stream);

        // Spawn a tokio task to serve multiple connections concurrently
        tokio::task::spawn(async move {
            // Finally, we bind the incoming connection to our `hello` service
            if let Err(err) = http1::Builder::new()
                // `service_fn` converts our function in a `Service`
                .serve_connection(io, service_fn(hello))
                .await
            {
                println!("Error serving connection: {:?}", err);
            }
        });
    }
}