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
use http_body_util::{Full, Empty};
use hyper::body::{Buf, Bytes};
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response};
use hyper_util::rt::TokioIo;
use hyper::header;
use tokio::net::TcpListener;
use hyper::body::Frame;
use hyper::{Method, StatusCode};
use http_body_util::{combinators::BoxBody, BodyExt};
use kairos_risc0_types::Deposit;
mod tests;

//let inputs: models::CircuitInputs = serde_json::from_reader(req.collect().await?.aggregate().reader())?;


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let host: [u8;4] = [127,0,0,1]; 
    let port: u16 = 3000;
    let addr = SocketAddr::from((host, port));

    // We create a TcpListener and bind it to 127.0.0.1:3000
    let listener = TcpListener::bind(addr).await?;

    println!("[Kairos API server] @ {:?}:{}", &host, &port);
    // We start a loop to continuously accept incoming connections
    loop {
        let (stream, _) = listener.accept().await?;

        // Use an adapter to access something implementing `tokio::io` traits as if they implement
        // `hyper::rt` IO traits.
        let io = TokioIo::new(stream);

        // Spawn a tokio task to serve multiple connections concurrently
        tokio::task::spawn(async move {
            // Finally, we bind the incoming connection to our `hello` service
            if let Err(err) = http1::Builder::new()
                // `service_fn` converts our function in a `Service`
                .serve_connection(io, service_fn(service))
                .await
            {
                println!("Error serving connection: {:?}", err);
            }
        });
    }
}

async fn service(
    req: Request<hyper::body::Incoming>,
) -> Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error> {
    match (req.method(), req.uri().path()) {
        (&Method::GET, "/") => Ok(Response::new(full(
            "Try submitting to /transfer",
        ))),

        (&Method::POST, "/transfer") => {
            let deposit: Deposit = serde_json::from_slice(&req.collect().await.unwrap().to_bytes()).unwrap();
            // for now this endpoint will simply echo the transfer,
            // later it should insert the transfer with processed=false into the DB table
            let response = Response::builder()
                .status(StatusCode::CREATED)
                .header(header::CONTENT_TYPE, "application/json")
                .body(full(serde_json::to_vec(&deposit).unwrap())).unwrap();
            
            Ok(response)
        },

        // Return 404 Not Found for other routes.
        _ => {
            let mut not_found = Response::new(empty());
            *not_found.status_mut() = StatusCode::NOT_FOUND;
            Ok(not_found)
        }
    }
}

// We create some utility functions to make Empty and Full bodies
// fit our broadened Response body type.
fn empty() -> BoxBody<Bytes, hyper::Error> {
    Empty::<Bytes>::new()
        .map_err(|never| match never {})
        .boxed()
}
fn full<T: Into<Bytes>>(chunk: T) -> BoxBody<Bytes, hyper::Error> {
    Full::new(chunk.into())
        .map_err(|never| match never {})
        .boxed()
}