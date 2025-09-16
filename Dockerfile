FROM rust:1.82-slim AS builder

ENV GRAALVM_VERSION=23.0.1
ENV GRAALVM_FILENAME=graalvm-community-jdk-23.0.1_linux-x64_bin.tar.gz
ENV GRAALVM_DOWNLOAD_URL=https://github.com/graalvm/graalvm-ce-builds/releases/download/jdk-23.0.1/${GRAALVM_FILENAME}

RUN apt-get update && apt-get install -y --no-install-recommends \
    poppler-utils \
    libssl-dev \
    pkg-config \
    build-essential \
    zlib1g-dev \
    wget \
    tar \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

RUN wget -O /tmp/${GRAALVM_FILENAME} ${GRAALVM_DOWNLOAD_URL} && \
    tar -xzf /tmp/${GRAALVM_FILENAME} -C /opt/ && \
    rm /tmp/${GRAALVM_FILENAME}

ENV JAVA_HOME=/opt/graalvm-community-jdk-${GRAALVM_VERSION}
ENV PATH=${JAVA_HOME}/bin:${PATH}

WORKDIR /usr/src/app
COPY Cargo.toml .
COPY src ./src

RUN cargo build --release

# Debug: List the contents of the build directory
RUN ls -lR /usr/src/app/target/release/build/

FROM debian:bookworm-slim AS runtime

ENV GRAALVM_VERSION=23.0.1
ENV GRAALVM_FILENAME=graalvm-community-jdk-23.0.1_linux-x64_bin.tar.gz
ENV GRAALVM_DOWNLOAD_URL=https://github.com/graalvm/graalvm-ce-builds/releases/download/jdk-23.0.1/${GRAALVM_FILENAME}

RUN apt-get update && apt-get install -y --no-install-recommends \
    poppler-utils \
    openssl \
    tesseract-ocr \
    tesseract-ocr-por \
    wget \
    tar \
    zlib1g \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

RUN wget -O /tmp/${GRAALVM_FILENAME} ${GRAALVM_DOWNLOAD_URL} && \
    tar -xzf /tmp/${GRAALVM_FILENAME} -C /opt/ && \
    rm /tmp/${GRAALVM_FILENAME}

ENV JAVA_HOME=/opt/graalvm-community-jdk-${GRAALVM_VERSION}
ENV PATH=${JAVA_HOME}/bin:${PATH}
ENV LD_LIBRARY_PATH=/usr/local/lib

WORKDIR /app

COPY --from=builder /usr/src/app/target/release/signature_extractor /usr/local/bin/
COPY --from=builder /usr/src/app/target/release/build/extractous-*/out/libs/*.so /usr/local/lib/

VOLUME /app/pdfs

ENTRYPOINT ["signature_extractor"]
CMD ["/app/pdfs"]
