# 1. Используем официальный образ Rust в качестве базового
FROM rust:latest AS build

# 2. Устанавливаем рабочую директорию в контейнере
WORKDIR /app

# 3. Копируем файлы Cargo.toml и Cargo.lock для кэширования зависимостей
COPY Cargo.toml Cargo.lock ./

# 4. Создаем пустой проект, чтобы закешировать зависимости
RUN mkdir src && echo "fn main() {}" > src/main.rs

# 5. Компилируем зависимости
RUN cargo build --release
RUN rm src/*.rs
ENV RUST_LOG=info
COPY ./src ./src
# build for release
RUN rm ./target/release/deps/*
RUN cargo build --release

FROM rust:latest

# copy the build artifact from the build stage
COPY --from=build /app/target/release/blockchain .
RUN mkdir "cache"
ENV RUST_LOG=debug
ENV RUST_BACKTRACE=full

EXPOSE 7878
# set the startup command to run your binary
CMD ["./blockchain"]

