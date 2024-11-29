FROM rust:1.78 as toolchain

WORKDIR /app

RUN curl --proto '=https' --tlsv1.2 -sSfL https://risczero.com/install > install-rzup.sh
RUN bash install-rzup.sh
ENV PATH="/root/.risc0/bin:/root/.cargo/bin:$PATH"
RUN mkdir -p /root/.cargo/bin && rzup toolchain install rust && rzup install cargo-risczero v1.0.1


FROM toolchain as build

# Copy files and build project. Cache rust packages and builds
COPY  ./ ./
RUN --mount=type=cache,target=target \
    cargo build --release && \
    cp target/release/zkEPDCalc /usr/local/bin/zkEPDCalc

# Copy the the zkEPDCalc and th r0vm binaries to the goth16 prover image
FROM docker.io/risczero/risc0-groth16-prover:v2024-05-17.1
# Add mock docker file to start the groth16 prover without launching an additional container
# See call in r0vm: https://github.com/risc0/risc0/blob/79de616506543634cb5d75b9db7f3aee3640d68c/risc0/groth16/src/docker.rs#L56
COPY --chmod=755 scripts/docker-wrapper.sh /usr/local/bin/docker
COPY --from=build /usr/local/bin/zkEPDCalc /root/.cargo/bin/r0vm /usr/local/bin/

RUN ulimit -s unlimited
WORKDIR /app
ENTRYPOINT ["/usr/local/bin/zkEPDCalc"]
