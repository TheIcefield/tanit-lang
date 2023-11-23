# Tanit compiler

## Description
Compiler of tanit language written in rust.

## Getting started

1. Clone repository:
```
git clone https://gitlab.com/oleg.icefield/tanitc.git
```

2. Build project:
```
cargo build --release
```

3. Compile your `*.tt` file:
```
cargo run -- -i examples/test.tt -o main
```

## Contributing
Contributers should just open a merge request, where pipeline shall analyze the changes for formatting, skipped unit tests, audit and additional lints with clippy. There should be no warnings in the project. 
