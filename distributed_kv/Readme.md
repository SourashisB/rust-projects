# Distributed Key-Value Store

This is a simple distributed key-value store implemented in Rust. It consists of a server and a command-line client.

## Features

- Simple key-value data structure
- Networking for distributed operations
- Basic consistency and replication (simulated)
- Command-line client

## Usage

### Running the server
cargo run -- server --address 127.0.0.1:8080


### Running the client
cargo run -- client --server 127.0.0.1:8080

### Client commands

- `GET <key>`: Retrieve the value for a given key
- `SET <key> <value>`: Set a value for a given key
- `DELETE <key>`: Delete a key-value pair
- `exit`: Exit the client

## Implementation Details

This project implements a basic distributed key-value store with the following components:

- Server: Handles incoming connections and processes commands
- Storage: Manages the key-value data structure
- Replication: Simulates data replication (not fully implemented)
- Client: Provides a command-line interface to interact with the server

Note: This is a simplified implementation and does not include advanced features like sharding, conflict resolution, or actual distributed consensus algorithms.