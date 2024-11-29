# zkEPDCalc

This is a EPD calculator service that comes with zero knowledge proofs.

The zero knowledge proof is used to create a EPD that can be verified without
needing to publish all the parameters involved in the calculations


## Quick Start

### Install Rust
First, make sure [rustup] is installed. The
[`rust-toolchain.toml`][rust-toolchain] file will be used by `cargo` to
automatically install the correct version.

### Install rist zero
Full manaul can be found [here](https://dev.risczero.com/api/zkvm/install)
```bash
cargo install cargo-binstall
cargo binstall cargo-risczero
cargo risczero install
```

### Run Server

To build all methods and execute the method within the zkVM, run the following
command:

```bash
cargo run
```
### Endpoints

The service provides two endpoints
- create: `POST` request that triggers the calculation of the epd and the generation of a zero konwledge proof

  parameters: 
  - `snark=[true|false]` calculate a short snark proof (default `false`) 
  - `zkType=[Concrete|BuildingPart|Building]` select the type of zkp to produce.
- creation: `POST` request that starts the calculation task in the background. Returns the id of the task
  
  parameters:
  - `snark=[true|false]` calculate a short snark proof (default `false`)
  - `zkType=[Concrete|BuildingPart|Building]` select the type of zkp to produce.
- creation/:id/ `GET` returns the status of the tasks (`completed` when finished)
- creation/:id/result `GET` returns the zero knowledge EPD.
- verify: `POST` request to verify a proof and check the commitments

  parameters:
  - `zkType=[Concrete|BuildingPart|Building]` select the type of zkp to verify.

Example usage with synchronous endpoint (long proof):
```bash
# Calculate a EPD with a snark proof 
epd=$(curl -X POST "http://127.0.0.1:3000/create?snark=false&zktype=Concrete" -H "Content-Type: application/json" -d '{
  "description": "C 25/30/B1",
  "cement": 124,
  "gravel": 775,
  "water": 75,
  "additives": 26,
  "material": "super starker Eco++",
  "factory": "Eggendorf"
}') && echo $epd
# verify EPD
curl "http://127.0.0.1:3000/verify&zktype=Concrete -H "Content-Type: application/json" -d $epd
```

Example usage with asynchronous endpoint (short proof, takes longer -> http timeout):
```bash
# Start the EPD calculation
task=$(curl -X POST "http://127.0.0.1:3000/creation?snark=true&zktype=Concrete" -H "Content-Type: application/json" -d '{
  "description": "C 25/30/B1",
  "cement": 124,
  "gravel": 775,
  "water": 75,
  "additives": 26,
  "material": "super starker Eco++",
  "factory": "Eggendorf"
}') && echo $task
# > {"id":"dc9847eb-0e8e-4e7b-a339-d08829b58ba8","state":"Submitted"}
id=$(echo $task | jq '.id')

# Check state of the creation
curl -X GET "http://127.0.0.1:3000/creation/$id"
# > {"id":"dc9847eb-0e8e-4e7b-a339-d08829b58ba8","state":"InProgress"} ⟲
# > {"id":"dc9847eb-0e8e-4e7b-a339-d08829b58ba8","state":"Complete"}
# If complete, retrieve zkEPD
curl -X GET "http://127.0.0.1:3000/creation/$id/result"
```
