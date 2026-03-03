# rtodo

a simple todo list cli

## Usage

```bash
cargo run -- add "buy milk"
cargo run -- add -p h --at now "finish report"
cargo run -- add --priority l --at 1710000000 "walk dog"
cargo run -- list
cargo run -- path              # print current todo storage file path
cargo run -- done 1
cargo run -- remove 1          # asks for double confirmation
cargo run -- remove 1 --yes    # skips confirmation
cargo run -- clear             # asks for double confirmation
cargo run -- clear --yes       # skips confirmation
```

`list` 默认按添加时间（added time）正序展示。

### Data storage

By default todos are stored in the user home directory at `~/.rtodo/todos.json`
(on Windows, prefers USERPROFILE/HOMEDRIVE+HOMEPATH over HOME to avoid shell-variable mismatch).
Override this with the `RTODO_FILE` environment variable.
