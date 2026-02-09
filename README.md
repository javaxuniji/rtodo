# rtodo

a simple todo list cli

## Usage

```bash
cargo run -- add "buy milk"
cargo run -- list
cargo run -- done 1
cargo run -- remove 1
cargo run -- clear
```

### Data storage

By default todos are stored in a `.rtodo.json` file in the current directory. Override
this with the `RTODO_FILE` environment variable.
