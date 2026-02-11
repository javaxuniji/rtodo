# rtodo

a simple todo list cli

## Usage

```bash
cargo run -- add "buy milk"
cargo run -- list
cargo run -- done 1
cargo run -- remove 1          # asks for double confirmation
cargo run -- remove 1 --yes    # skips confirmation
cargo run -- clear             # asks for double confirmation
cargo run -- clear --yes       # skips confirmation
```

### Data storage

By default todos are stored in `~/.rtodo/todos.json`. Override this with the
`RTODO_FILE` environment variable.
