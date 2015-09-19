FROM debian:jessie

RUN apt-get update && apt-get install --fix-missing -y clang

RUN apt-get update && apt-get install -y curl file
RUN apt-get update && apt-get install -y libssl-dev

# The rust installation script uses the sudo command which
# doesn't exist for the root user. Here's a workaround
RUN echo '"$@"' > /usr/bin/sudo && chmod +x /usr/bin/sudo

RUN curl -sSf https://static.rust-lang.org/rustup.sh | bash -s -- -y

RUN apt-get update && apt-get install --yes git

RUN mkdir /root/.cargo

WORKDIR /code
