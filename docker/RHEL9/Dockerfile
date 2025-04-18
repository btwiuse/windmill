ARG DEBIAN_IMAGE=debian:bookworm-slim
ARG RUST_IMAGE=registry.access.redhat.com/ubi9/ubi:latest
ARG PYTHON_IMAGE=python:3.11.10-slim-bookworm

FROM ${RUST_IMAGE} AS rust_base

RUN yum update -y && \
    yum install -y git openssl-devel npm nodejs rustfmt

# Install rust manually
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

RUN CARGO_NET_GIT_FETCH_WITH_CLI=true cargo install cargo-chef --version ^0.1

WORKDIR /windmill

ENV SQLX_OFFLINE=true
# ENV CARGO_INCREMENTAL=1

FROM node:20-alpine as frontend

# install dependencies
WORKDIR /frontend
COPY ./frontend/package.json ./frontend/package-lock.json ./
RUN npm ci

# Copy all local files into the image.
COPY frontend .
RUN mkdir /backend
COPY /backend/windmill-api/openapi.yaml /backend/windmill-api/openapi.yaml
COPY /openflow.openapi.yaml /openflow.openapi.yaml
COPY /backend/windmill-api/build_openapi.sh /backend/windmill-api/build_openapi.sh

RUN cd /backend/windmill-api && . ./build_openapi.sh
COPY /backend/parsers/windmill-parser-wasm/pkg/ /backend/parsers/windmill-parser-wasm/pkg/
COPY /typescript-client/docs/ /frontend/static/tsdocs/

RUN npm run generate-backend-client
ENV NODE_OPTIONS "--max-old-space-size=10240"
RUN npm run build


FROM rust_base AS planner

COPY ./openflow.openapi.yaml /openflow.openapi.yaml
COPY ./backend ./

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    CARGO_NET_GIT_FETCH_WITH_CLI=true cargo chef prepare --recipe-path recipe.json

FROM rust_base AS builder
ARG features=""

COPY --from=planner /windmill/recipe.json recipe.json

RUN --mount=type=secret,id=rh_username \
    --mount=type=secret,id=rh_password \
    subscription-manager register --username $(cat /run/secrets/rh_username) --password $(cat /run/secrets/rh_password)

RUN subscription-manager repos --enable codeready-builder-for-rhel-9-$(arch)-rpms

RUN yum update -y && \
    yum install -y perl-FindBin perl-IPC-Cmd libxml2-devel xmlsec1-devel xmlsec1-openssl-devel clang llvm-devel cmake libtool-ltdl-devel

# RUN --mount=type=cache,target=/usr/local/cargo/registry \
#     CARGO_NET_GIT_FETCH_WITH_CLI=true RUST_BACKTRACE=1 cargo chef cook --release --features "$features" --recipe-path recipe.json

COPY ./openflow.openapi.yaml /openflow.openapi.yaml
COPY ./backend ./

COPY --from=frontend /frontend /frontend
COPY --from=frontend /backend/windmill-api/openapi-deref.yaml ./windmill-api/openapi-deref.yaml
COPY .git/ .git/

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    CARGO_NET_GIT_FETCH_WITH_CLI=true cargo build --release --features "$features"

RUN subscription-manager unregister
