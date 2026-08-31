FROM rust:1.98.0-trixie

ENV DEBIAN_FRONTEND=noninteractive \
    CARGO_HOME=/usr/local/cargo \
    RUSTUP_HOME=/usr/local/rustup \
    PATH=/usr/local/cargo/bin:${PATH}

ARG BAZELISK_VERSION=1.29.0
ARG NODE_VERSION=24.20.0
ARG TYPESCRIPT_VERSION=7.0.2
ARG PRETTIER_VERSION=3.9.6
ARG RUFF_VERSION=0.16.5
ARG MYPY_VERSION=2.3.1
ARG PYTEST_VERSION=9.1.1

RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        bash \
        build-essential \
        ca-certificates \
        clang \
        curl \
        file \
        git \
        less \
        pkg-config \
        python3 \
        python3-venv \
        unzip \
        vim-tiny \
        xz-utils \
        zip \
    && rm -rf /var/lib/apt/lists/*

RUN rustup component add clippy rustfmt

RUN python3 -m venv /opt/polyrust-python-tools \
    && /opt/polyrust-python-tools/bin/pip install --no-cache-dir \
        "ruff==${RUFF_VERSION}" \
        "mypy==${MYPY_VERSION}" \
        "pytest==${PYTEST_VERSION}" \
    && ln -s /opt/polyrust-python-tools/bin/ruff /usr/local/bin/ruff \
    && ln -s /opt/polyrust-python-tools/bin/mypy /usr/local/bin/mypy \
    && ln -s /opt/polyrust-python-tools/bin/pytest /usr/local/bin/pytest

RUN set -eux; \
    curl --fail --silent --show-error --location \
        --output /tmp/node.tar.xz \
        "https://nodejs.org/dist/v${NODE_VERSION}/node-v${NODE_VERSION}-linux-x64.tar.xz"; \
    curl --fail --silent --show-error --location \
        --output /tmp/node-shasums.txt \
        "https://nodejs.org/dist/v${NODE_VERSION}/SHASUMS256.txt"; \
    expected_checksum=$(awk -v file="node-v${NODE_VERSION}-linux-x64.tar.xz" '$2 == file { print $1 }' /tmp/node-shasums.txt); \
    actual_checksum=$(sha256sum /tmp/node.tar.xz | cut -d ' ' -f 1); \
    test -n "$expected_checksum"; \
    test "$expected_checksum" = "$actual_checksum"; \
    tar -xJf /tmp/node.tar.xz -C /usr/local --strip-components=1; \
    npm install --global --ignore-scripts \
        "typescript@${TYPESCRIPT_VERSION}" \
        "prettier@${PRETTIER_VERSION}"; \
    node --version; \
    npm --version; \
    tsc --version; \
    prettier --version; \
    rm -f /tmp/node.tar.xz /tmp/node-shasums.txt

RUN set -eux; \
    curl --fail --silent --show-error --location \
        --output /tmp/bazelisk-linux-amd64 \
        "https://github.com/bazelbuild/bazelisk/releases/download/v${BAZELISK_VERSION}/bazelisk-linux-amd64"; \
    curl --fail --silent --show-error --location \
        --output /tmp/bazelisk-linux-amd64.sha256 \
        "https://github.com/bazelbuild/bazelisk/releases/download/v${BAZELISK_VERSION}/bazelisk-linux-amd64.sha256"; \
    expected_checksum=$(tr -d '[:space:]' < /tmp/bazelisk-linux-amd64.sha256); \
    actual_checksum=$(sha256sum /tmp/bazelisk-linux-amd64 | cut -d ' ' -f 1); \
    test "$expected_checksum" = "$actual_checksum"; \
    install -m 0755 /tmp/bazelisk-linux-amd64 /usr/local/bin/bazelisk; \
    ln -s /usr/local/bin/bazelisk /usr/local/bin/bazel; \
    rm -f /tmp/bazelisk-linux-amd64 /tmp/bazelisk-linux-amd64.sha256

# Bazel downloads the pinned Go SDK declared in MODULE.bazel. The same SDK is
# available for ad-hoc commands through `bazel run @io_bazel_rules_go//go`.
RUN printf '%s\n' 'export PATH=/usr/local/cargo/bin:$PATH' > /etc/profile.d/rust.sh

WORKDIR /workspace
CMD ["sleep", "infinity"]
