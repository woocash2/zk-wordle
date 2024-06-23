# ZK-Wordle

Zero-knowledge wordle game which protects players from dishonest servers.

## Running the server

Just hit
```
cargo run
```
in the `server` directory. Recommended to use env `RUST_LOG=info,wasmer_compiler_cranelift=warn` to see the meaningful logs, but hide some spammy library logs. You can verify the server working by example curls:
```
curl http://localhost:4000/start
```
or
```
curl http://localhost:4000/guess -H 'Content-Type: application/json' -d '{"word_id": 0, "guess": "hello"}'
```

## Frontend

We have da frontend application in the `front` directory.

## Circuits

`proof-clue` and `proof-membership` contain circuit descriptions in `*.circom` files. The data generated from them is already in the repository (in `proof-clue, proof-membership, keys` directories), but if you want to compile them again, you need to `mkdir circomlib` in the root directory of the project, download `circuits/poseidon.circom, circuits/poseidon_constants.circom` from circomlib: https://github.com/iden3/circomlib, put them in the created directory, and then you should be able to compile them. (**not recommended and not needed but possible**).