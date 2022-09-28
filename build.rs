fn main() {
    tonic_build::configure()
        .build_client(false)
        .build_server(true)
        .compile(&["proto/orderbook.proto"], &[] as &[&str])
        .expect("Error: failed to compile proto file");
}
