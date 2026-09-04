#!/bin/sh
set -e

TESTER_BIN="${SHELL_TESTER_BIN:-$HOME/dev/shell-tester/dist/main.out}"
REPO_DIR=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
MAX="${1:-0}"

usage() {
    printf 'usage: %s [number-of-stages]\n' "$0" >&2
    exit 1
}

if [ ! -x "$TESTER_BIN" ]; then
    printf 'error: tester binary not found at %s\n' "$TESTER_BIN" >&2
    printf 'set it up with:\n' >&2
    printf '  git clone --depth 1 https://github.com/codecrafters-io/shell-tester ~/dev/shell-tester\n' >&2
    printf '  make -C ~/dev/shell-tester build\n' >&2
    printf 'or point SHELL_TESTER_BIN at an existing build\n' >&2
    exit 1
fi

case $MAX in
    ''|*[!0-9]*) usage ;;
esac

if [ -d "$HOME/.rustup" ]; then
    : "${RUSTUP_HOME:=$HOME/.rustup}"
    export RUSTUP_HOME
fi
if [ -d "$HOME/.cargo" ]; then
    : "${CARGO_HOME:=$HOME/.cargo}"
    export CARGO_HOME
fi

STAGE_LIST=$(cat <<'EOF'
oo8|Print a prompt
cz2|Handle invalid commands
ff0|REPL
pn5|The exit builtin
iz3|The echo builtin
ez5|Type builtin: builtins
mg5|Type builtin: executables
ip1|Run a program
ei0|The pwd builtin
ra6|cd builtin: absolute paths
gq9|cd builtin: relative paths
gp4|cd builtin: home directory
ni6|Single quotes
tg6|Double quotes
yt5|Backslash outside quotes
le5|Backslash within single quotes
gu3|Backslash within double quotes
qj0|Executing a quoted executable
jv1|Redirect stdout
vz4|Redirect stderr
el9|Append stdout
un3|Append stderr
qp2|Builtin completion
gm9|Completion with arguments
qm8|Missing completions
gy5|Executable completion
wh6|Multiple completions
wt6|Partial completions
zv2|File completion
ue6|Nested file completion
lc6|Directory completion
vs5|Missing entry completion
no5|Multiple matches
jp8|Partial filename completions
bf8|Multi-argument completions
ne7|Register complete builtin
oi7|Printing missing specifications
wl6|Displaying registered specifications
pm5|Single completion
qf1|Handling no completions
zi0|Passing command-line arguments
nr7|Passing environment variables
ep2|Multiple completer candidates
xz3|Longest common prefix
tz2|Unregister a completion
af3|The jobs builtin
at7|Starting background jobs
si2|Printing background job output
jd6|List a single job
dk5|List multiple jobs
ma9|Reaping one job using jobs
rq2|Reaping multiple jobs using jobs
bv8|Reap before the next prompt
fy4|Job number reset
br6|Dual-command pipeline
ny9|Pipelines with built-ins
xk3|Multi-command pipelines
bq4|The history builtin
yf5|Listing history
ag6|Limiting history entries
rh7|UP arrow navigation
vq0|DOWN arrow navigation
dm2|Enter with arrow navigation
za2|Read history from file
in3|Write history to file
sx3|Append history to file
zp4|Read history on startup
kz7|Write history on exit
jv2|Append history on exit
ji0|The declare builtin
oa2|Printing non-existing variables
kv5|Storing shell variables
db8|Validating shell variable names
ge9|Expanding variables
br2|Expansion with braces
my0|Expansion when a name is unset
EOF
)

total=0
selected=0
json="["
while IFS="|" read -r slug title; do
    if [ -z "$slug" ]; then
        continue
    fi
    total=$((total + 1))
    if [ "$MAX" -gt 0 ] && [ "$total" -gt "$MAX" ]; then
        continue
    fi
    selected=$((selected + 1))
    if [ "$selected" -gt 1 ]; then
        json="$json,"
    fi
    json="$json{\"slug\":\"$slug\",\"tester_log_prefix\":\"tester::#$slug\",\"title\":\"$title\"}"
done <<EOF
$STAGE_LIST
EOF
json="$json]"

if [ "$selected" -eq 0 ]; then
    usage
fi

printf 'Running %d of %d stages\n\n' "$selected" "$total"

cargo build --release --target-dir=/tmp/codecrafters-build-shell-rust --manifest-path "$REPO_DIR/Cargo.toml" >/dev/null

cd "$REPO_DIR"
exec env CODECRAFTERS_REPOSITORY_DIR="$REPO_DIR" CODECRAFTERS_TEST_CASES_JSON="$json" "$TESTER_BIN"
