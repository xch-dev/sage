#!/usr/bin/env bash
#
# Sage RPC Smoke Test — testnet11 end-to-end
#
# Runs two Sage RPC servers with separate data dirs against testnet11,
# creates real blockchain artifacts (DID, CAT, NFT, offer), and verifies them.
#
# Usage:
#   ./scripts/smoke_test.sh [--timeout SECONDS] [--return-address ADDR] [--verbose]
#
# Exit codes:
#   0  all tests passed
#   1  test assertion failed
#   2  infrastructure failure

set -euo pipefail

# ─── Configuration ──────────────────────────────────────────────────────────
PORT_A=19257
PORT_B=19258
TIMEOUT=300             # seconds for all waits (funding, confirmations, sync)
VERBOSE=false
PASSED=0
FAILED=0
RETURN_ADDRESS=""
SAGE_BIN=""
DIR_A=""
DIR_B=""
PID_A=""
PID_B=""

# ─── Argument parsing ──────────────────────────────────────────────────────
while [[ $# -gt 0 ]]; do
    case "$1" in
        --timeout)          TIMEOUT="$2"; shift 2 ;;
        --return-address)   RETURN_ADDRESS="$2"; shift 2 ;;
        --verbose|-v)       VERBOSE=true; shift ;;
        *)                  echo "Unknown option: $1"; exit 2 ;;
    esac
done

# Prompt for return address if not provided (skip in non-interactive mode)
if [[ -z "$RETURN_ADDRESS" && -t 0 ]]; then
    echo ""
    echo "Enter a testnet11 address to return remaining TXCH after the test"
    echo "(leave blank to skip fund return):"
    read -r RETURN_ADDRESS
    echo ""
fi

# ─── Helpers ────────────────────────────────────────────────────────────────

BLUE='\033[1;34m'
GREEN='\033[1;32m'
RED='\033[1;31m'
YELLOW='\033[1;33m'
DIM='\033[2m'
RESET='\033[0m'

# File descriptor 3 = real terminal. Progress dots write to fd 3 so they
# remain visible even inside subshells whose stdout is redirected to temp files.
exec 3>&1

PHASE_START=0

log()       { echo "    $*"; }
info()      { echo "  🔹 $*"; }
action()    { echo "    ⏳ $*"; }
log_start() {
    PHASE_START=$SECONDS
    echo ""
    echo -e "${DIM}────────────────────────────────────────────────────────────${RESET}"
    echo -e "${BLUE}$*${RESET}"
    echo -e "${DIM}────────────────────────────────────────────────────────────${RESET}"
}
log_done()  {
    local elapsed=$(( SECONDS - PHASE_START ))
    echo -e "\n${GREEN}$*${RESET} ${DIM}(${elapsed}s)${RESET}"
}
warn()      { echo "  ⚠️  $*" >&2; }
die()       { echo "  💀 $*" >&2; exit 2; }

cleanup() {
    local exit_code=$?
    echo ""
    echo -e "${DIM}════════════════════════════════════════════════════════════${RESET}"
    if [[ $FAILED -gt 0 || $exit_code -ne 0 ]]; then
        echo -e "  🏁 Results: ${GREEN}$PASSED passed${RESET}, ${RED}$FAILED failed${RESET}"
    else
        echo -e "  🏁 Results: ${GREEN}$PASSED passed${RESET}, 0 failed"
    fi
    echo -e "${DIM}════════════════════════════════════════════════════════════${RESET}"

    if [[ $FAILED -gt 0 || $exit_code -ne 0 ]]; then
        echo ""
        echo -e "  ${YELLOW}🔒 Preserving environment for debugging${RESET}"
        echo "  Servers are still running. To probe:"
        [[ -n "$PID_A" ]] && echo "    Alice: PID $PID_A, port $PORT_A, data dir: $DIR_A"
        [[ -n "$PID_B" ]] && echo "    Bob:   PID $PID_B, port $PORT_B, data dir: $DIR_B"
        [[ -n "$DIR_A" ]] && echo "    Alice log: $DIR_A/sage.log"
        [[ -n "$DIR_B" ]] && echo "    Bob log:   $DIR_B/sage.log"
        echo ""
        echo "  To clean up manually:"
        echo "    kill $PID_A $PID_B 2>/dev/null; rm -rf $DIR_A $DIR_B"
        echo ""
    else
        action "Cleaning up..."
        [[ -n "$PID_A" ]] && kill "$PID_A" 2>/dev/null || true
        [[ -n "$PID_B" ]] && kill "$PID_B" 2>/dev/null || true
        sleep 1
        [[ -n "$PID_A" ]] && kill -9 "$PID_A" 2>/dev/null || true
        [[ -n "$PID_B" ]] && kill -9 "$PID_B" 2>/dev/null || true
        # Fallback: kill anything still holding our ports
        if command -v lsof &>/dev/null; then
            for port in "$PORT_A" "$PORT_B"; do
                pid=$(lsof -ti :"$port" 2>/dev/null) || true
                [[ -n "$pid" ]] && kill -9 "$pid" 2>/dev/null || true
            done
        fi
        [[ -n "$DIR_A" ]] && rm -rf "$DIR_A"
        [[ -n "$DIR_B" ]] && rm -rf "$DIR_B"
    fi
}
trap cleanup EXIT

# mTLS curl wrapper: curl_rpc DIR PORT ENDPOINT JSON_BODY
curl_rpc() {
    local dir="$1" port="$2" endpoint="$3" body
    body="${4:-}"; body="${body:-{}}"
    local http_code response
    response=$(curl -s -w '\n%{http_code}' \
        --cert "$dir/ssl/wallet.crt" \
        --key  "$dir/ssl/wallet.key" \
        --cacert "$dir/ssl/wallet.crt" \
        -H "Content-Type: application/json" \
        -d "$body" \
        "https://127.0.0.1:${port}/${endpoint}" \
        --insecure 2>/dev/null) || { echo '{"error":"curl_failed"}'; return 1; }
    http_code=$(echo "$response" | tail -1)
    response=$(echo "$response" | sed '$d')
    if [[ "$http_code" -ge 400 && "$VERBOSE" == "true" ]] 2>/dev/null; then
        warn "HTTP $http_code from $endpoint: $response"
    fi
    echo "$response"
}

rpc_a() { curl_rpc "$DIR_A" "$PORT_A" "$@"; }
rpc_b() { curl_rpc "$DIR_B" "$PORT_B" "$@"; }

# Wait for server to respond to get_version (up to 60s)
wait_for_server() {
    local dir="$1" port="$2" elapsed=0
    action "Waiting for server on port $port..."
    while ! curl_rpc "$dir" "$port" "get_version" '{}' >/dev/null 2>&1; do
        sleep 2
        elapsed=$((elapsed + 2))
        if [[ $elapsed -ge $TIMEOUT ]]; then
            die "Server on port $port did not start within ${TIMEOUT}s"
        fi
    done
    info "Server on port $port is ready (${elapsed}s)"
}

# Wait for wallet balance >= MIN mojos (with dot animation)
wait_for_balance() {
    local dir="$1" port="$2" min="$3" label="${4:-Waiting for funding}" elapsed=0
    printf "    ⏳ %s " "$label" >&3
    while true; do
        local json bal
        json=$(curl_rpc "$dir" "$port" "get_sync_status" '{}' 2>/dev/null) || true
        bal=$(echo "$json" | jq -r '.balance // "0"' 2>/dev/null) || bal="0"
        if [[ "$bal" -ge "$min" ]] 2>/dev/null; then
            printf " ✅ (%ds, %s mojos)\n" "$elapsed" "$bal" >&3
            return 0
        fi
        # Log first response for debugging
        if [[ $elapsed -eq 0 && "$VERBOSE" == "true" ]]; then
            printf "\n    [debug] get_sync_status returned: %s\n" "$(echo "$json" | jq -c '.' 2>/dev/null || echo "$json")" >&3
        fi
        printf "." >&3
        sleep 5
        elapsed=$((elapsed + 5))
        if [[ $elapsed -ge $TIMEOUT ]]; then
            printf " ❌ (timeout)\n" >&3
            warn "Timed out waiting for balance >= $min mojos after ${TIMEOUT}s (last response: $(echo "$json" | jq -c '.' 2>/dev/null || echo "$json"))"
            return 1
        fi
    done
}

# Wait for pending transactions to clear (with dot animation)
wait_for_pending_clear() {
    local dir="$1" port="$2" label="${3:-Waiting for next block}" elapsed=0
    printf "    ⏳ %s " "$label" >&3
    while true; do
        local json count
        json=$(curl_rpc "$dir" "$port" "get_pending_transactions" '{}' 2>/dev/null) || true
        if [[ -n "$json" ]]; then
            count=$(echo "$json" | jq '.transactions | length' 2>/dev/null) || count=999
            if [[ "$count" -eq 0 ]]; then
                printf " ✅ (%ds)\n" "$elapsed" >&3
                return 0
            fi
        fi
        printf "." >&3
        sleep 5
        elapsed=$((elapsed + 5))
        if [[ $elapsed -ge $TIMEOUT ]]; then
            printf " ❌ (timeout)\n" >&3
            warn "Pending transactions did not clear within ${TIMEOUT}s"
            return 1
        fi
    done
}

# Wait for an asset to appear on a wallet (brief sync poll, not a blockchain wait)
wait_for_sync() {
    local endpoint="$1" filter="$2" dir="$3" port="$4" label="$5" body max
    body="${6:-}"; body="${body:-{}}"
    max="${7:-}"; max="${max:-$TIMEOUT}"
    local elapsed=0
    printf "    ⏳ Waiting for %s " "$label" >&3
    while true; do
        local json val
        json=$(curl_rpc "$dir" "$port" "$endpoint" "$body" 2>/dev/null) || true
        val=$(echo "$json" | jq -r "$filter" 2>/dev/null) || val="0"
        if [[ "$val" -ge 1 ]] 2>/dev/null; then
            printf " ✅ (%ds)\n" "$elapsed" >&3
            echo "$json"
            return 0
        fi
        # Log first response for debugging
        if [[ $elapsed -eq 0 && "$VERBOSE" == "true" ]]; then
            printf "\n    [debug] %s returned: %s\n" "$endpoint" "$(echo "$json" | jq -c '.' 2>/dev/null || echo "$json")" >&3
        fi
        printf "." >&3
        sleep 5
        elapsed=$((elapsed + 5))
        if [[ $elapsed -ge $max ]]; then
            printf " ❌ (timeout)\n" >&3
            warn "$label not detected after ${max}s (last response: $(echo "$json" | jq -c '.' 2>/dev/null || echo "$json"))"
            echo "$json"
            return 1
        fi
    done
}

# Assertion helpers (modifies global PASSED/FAILED — use only in main script)
assert_jq() {
    local label="$1" json="$2" filter="$3" expected="$4"
    local actual
    actual=$(echo "$json" | jq -r "$filter" 2>/dev/null) || actual="<jq error>"
    if [[ "$actual" == "$expected" ]]; then
        echo "  ✅ $label"
        PASSED=$((PASSED + 1))
    else
        echo "  ❌ $label (expected: $expected, got: $actual)"
        FAILED=$((FAILED + 1))
    fi
}

assert_jq_gte() {
    local label="$1" json="$2" filter="$3" min="$4"
    local actual
    actual=$(echo "$json" | jq -r "$filter" 2>/dev/null) || actual="0"
    if [[ "$actual" -ge "$min" ]] 2>/dev/null; then
        echo "  ✅ $label (value: $actual)"
        PASSED=$((PASSED + 1))
    else
        echo "  ❌ $label (expected >= $min, got: $actual)"
        FAILED=$((FAILED + 1))
    fi
}

# Subshell-safe assertion (uses local P/F counters initialized within each subshell)
check() {
    local label="$1" ok="$2"
    if [[ "$ok" == "true" ]]; then
        echo "  ✅ $label"; P=$((P + 1))
    else
        echo "  ❌ $label"; F=$((F + 1))
    fi
}

# Subshell-safe assertion for void endpoints (valid JSON = success)
check_ok() {
    local label="$1" result="$2"
    if echo "$result" | jq -e '.' >/dev/null 2>&1; then
        echo "  ✅ $label"; P=$((P + 1))
    else
        echo "  ❌ $label (got: $result)"; F=$((F + 1))
    fi
}

# Wait for two parallel subshells, collect their output and P/F counts.
# Usage: collect_parallel PID_A PID_B RESULT_A RESULT_B OUT_A OUT_B
collect_parallel() {
    local pid_a="$1" pid_b="$2" res_a="$3" res_b="$4" out_a="$5" out_b="$6"
    wait "$pid_a" 2>/dev/null || true
    wait "$pid_b" 2>/dev/null || true
    cat "$out_a"
    cat "$out_b"
    local pa fa pb fb
    read -r pa fa < "$res_a" 2>/dev/null || { pa=0; fa=0; }
    read -r pb fb < "$res_b" 2>/dev/null || { pb=0; fb=0; }
    PASSED=$((PASSED + pa + pb))
    FAILED=$((FAILED + fa + fb))
    rm -f "$res_a" "$res_b" "$out_a" "$out_b"
}

# Return remaining XCH from a wallet to an address.
# Usage: return_wallet_funds RPC_FN LABEL
return_wallet_funds() {
    local rpc_fn="$1" label="$2"
    local bal fee amt
    bal=$("$rpc_fn" "get_sync_status" '{}' | jq -r '.balance // "0"') || bal="0"
    if [[ "$bal" -gt 1000 ]] 2>/dev/null; then
        fee=$(( (RANDOM % 1000) + 1 ))
        amt=$((bal - fee))
        action "Returning $amt mojos from $label (fee: $fee)..."
        "$rpc_fn" "send_xch" "$(jq -n --arg addr "$RETURN_ADDRESS" --arg amt "$amt" --arg fee "$fee" '{
            address: $addr, amount: $amt, fee: $fee, memos: [], auto_submit: true
        }')" >/dev/null 2>&1 || warn "Failed to return funds from $label"
    elif [[ "$bal" -gt 0 ]] 2>/dev/null; then
        action "Returning $bal mojos from $label (fee: 0, balance too small for fee)..."
        "$rpc_fn" "send_xch" "$(jq -n --arg addr "$RETURN_ADDRESS" --arg amt "$bal" '{
            address: $addr, amount: $amt, fee: "0", memos: [], auto_submit: true
        }')" >/dev/null 2>&1 || warn "Failed to return funds from $label"
    fi
}

# ═══════════════════════════════════════════════════════════════════════════
# BLOCKCHAIN PHASES — each: submit → single wait → verify
# ═══════════════════════════════════════════════════════════════════════════

# ─── Setup ─────────────────────────────────────────────────────────────────
log_start "📦 Setup"

# Kill any leftover processes on our ports from a previous run
if command -v lsof &>/dev/null; then
    for port in "$PORT_A" "$PORT_B"; do
        pid=$(lsof -ti :"$port" 2>/dev/null) || true
        if [[ -n "$pid" ]]; then
            action "Killing leftover process on port $port (PID $pid)..."
            kill "$pid" 2>/dev/null || true
            sleep 1
            kill -9 "$pid" 2>/dev/null || true
        fi
    done
fi

# Find cargo (may not be on PATH)
CARGO="${CARGO:-$(command -v cargo 2>/dev/null || echo "$HOME/.cargo/bin/cargo")}"

# Find or build the sage binary
if [[ -f "target/release/sage" ]]; then
    SAGE_BIN="$(pwd)/target/release/sage"
    info "Using existing release binary: $SAGE_BIN"
else
    action "Building sage-cli (release)..."
    "$CARGO" build --release -p sage-cli || die "cargo build failed"
    SAGE_BIN="$(pwd)/target/release/sage"
fi

[[ -x "$SAGE_BIN" ]] || die "sage-cli binary not found at $SAGE_BIN"

# Create temp data directories
DIR_A=$(mktemp -d "${TMPDIR:-/tmp}/sage-smoke-alice.XXXXXX")
DIR_B=$(mktemp -d "${TMPDIR:-/tmp}/sage-smoke-bob.XXXXXX")
info "Alice data dir: $DIR_A"
info "Bob data dir:   $DIR_B"
info "Alice log: $DIR_A/log/app.log"
info "Bob log:   $DIR_B/log/app.log"

# Pre-write config.toml for each wallet
for dir_port in "$DIR_A:$PORT_A" "$DIR_B:$PORT_B"; do
    dir="${dir_port%%:*}"
    port="${dir_port##*:}"
    cat > "$dir/config.toml" <<EOF
version = 2

[global]
log_level = "INFO"

[network]
default_network = "testnet11"
target_peers = 5
discover_peers = true

[rpc]
enabled = true
port = $port
EOF
done

# Pre-write wallets.toml (empty wallet list — required for setup_config)
for dir in "$DIR_A" "$DIR_B"; do
    cat > "$dir/wallets.toml" <<EOF
[defaults]
delta_sync = true
EOF
done

info "Config files written"

# Start RPC servers (debug logging configured via config.toml log_level)
info "Starting Alice's RPC server (port $PORT_A)"
if [[ "$VERBOSE" == "true" ]]; then
    "$SAGE_BIN" --data-dir "$DIR_A" rpc start &
else
    "$SAGE_BIN" --data-dir "$DIR_A" rpc start >/dev/null 2>&1 &
fi
PID_A=$!

info "Starting Bob's RPC server (port $PORT_B)"
if [[ "$VERBOSE" == "true" ]]; then
    "$SAGE_BIN" --data-dir "$DIR_B" rpc start &
else
    "$SAGE_BIN" --data-dir "$DIR_B" rpc start >/dev/null 2>&1 &
fi
PID_B=$!

# Wait for both servers
wait_for_server "$DIR_A" "$PORT_A"
wait_for_server "$DIR_B" "$PORT_B"

# Verify get_version
json=$(rpc_a "get_version" '{}')
assert_jq "Alice calls [get_version] successfully" "$json" '.version | length > 0' "true"

log_done "✅ Both servers running"

# ─── Wallet setup ──────────────────────────────────────────────────────────
log_start "🔑 Setting up wallets for Alice and Bob"

# Generate mnemonics
MNEMONIC_A=$(rpc_a "generate_mnemonic" '{"use_24_words": true}' | jq -r '.mnemonic')
MNEMONIC_B=$(rpc_b "generate_mnemonic" '{"use_24_words": true}' | jq -r '.mnemonic')
info "Alice mnemonic: $MNEMONIC_A"
info "Bob mnemonic: $MNEMONIC_B"

# Import keys
IMPORT_A=$(rpc_a "import_key" "$(jq -n --arg m "$MNEMONIC_A" '{
    name: "Alice",
    key: $m,
    save_secrets: true,
    login: true
}')")
FP_A=$(echo "$IMPORT_A" | jq -r '.fingerprint')
info "Alice fingerprint: $FP_A"

IMPORT_B=$(rpc_b "import_key" "$(jq -n --arg m "$MNEMONIC_B" '{
    name: "Bob",
    key: $m,
    save_secrets: true,
    login: true
}')")
FP_B=$(echo "$IMPORT_B" | jq -r '.fingerprint')
info "Bob fingerprint: $FP_B"

# Wait for wallets to generate derivations and produce receive addresses
ADDR_A="" ADDR_B=""
for i in $(seq 1 30); do
    [[ -z "$ADDR_A" || "$ADDR_A" == "null" ]] && \
        ADDR_A=$(rpc_a "get_sync_status" '{}' | jq -r '.receive_address // empty' 2>/dev/null) || true
    [[ -z "$ADDR_B" || "$ADDR_B" == "null" ]] && \
        ADDR_B=$(rpc_b "get_sync_status" '{}' | jq -r '.receive_address // empty' 2>/dev/null) || true
    [[ -n "$ADDR_A" && "$ADDR_A" != "null" && -n "$ADDR_B" && "$ADDR_B" != "null" ]] && break
    sleep 2
done
[[ -n "$ADDR_A" && "$ADDR_A" != "null" ]] || die "Alice has no receive address after 60s"
[[ -n "$ADDR_B" && "$ADDR_B" != "null" ]] || die "Bob has no receive address after 60s"
info "Alice address: $ADDR_A"
info "Bob address: $ADDR_B"

log_done "✅ Wallets created for Alice and Bob"

# ─── Fund Alice ────────────────────────────────────────────────────────────
log_start "💰 Funding Alice's wallet"

info "Please send testnet11 TXCH to:"
info "  Alice address: $ADDR_A"
info "  Bob address funded by Alice"
info "Minimum: 0.000005 TXCH (5000000 mojos)."
info "Waiting up to ${TIMEOUT}s for funds to arrive..."

wait_for_balance "$DIR_A" "$PORT_A" 5000000 "Waiting for Alice to receive TXCH" || die "Alice not funded"

log_done "✅ Alice's wallet funded"

# ─── Split Alice's coin ───────────────────────────────────────────────────
log_start "✂️  Alice splits her coin into 100 for parallel transactions + pagination"

SPLIT_COINS=$(rpc_a "get_coins" '{"offset": 0, "limit": 1, "sort_mode": "amount", "ascending": false}')
SPLIT_COIN_ID=$(echo "$SPLIT_COINS" | jq -r '.coins[0].coin_id // ""' 2>/dev/null) || SPLIT_COIN_ID=""
[[ -n "$SPLIT_COIN_ID" && "$SPLIT_COIN_ID" != "null" ]] || die "No coin available for split"

SPLIT_RESULT=$(rpc_a "split" "$(jq -n --arg id "$SPLIT_COIN_ID" '{
    coin_ids: [$id], output_count: 100, fee: "100", auto_submit: true
}')")
assert_jq "Alice calls [split] successfully" "$SPLIT_RESULT" '.summary | length > 0' "true"

wait_for_pending_clear "$DIR_A" "$PORT_A" "Waiting for Alice's split to confirm"

COIN_COUNT=$(rpc_a "get_spendable_coin_count" '{}' | jq '.count' 2>/dev/null) || COIN_COUNT=0
info "Alice now has $COIN_COUNT coins"

# Pagination test (best time: Alice has ~100 clean coins)
PAGE1=$(rpc_a "get_coins" '{"offset": 0, "limit": 25, "sort_mode": "amount", "ascending": false}')
TOTAL_COINS=$(echo "$PAGE1" | jq '.total' 2>/dev/null) || TOTAL_COINS=0
assert_jq_gte "Alice calls [get_coins] and has ≥100 after split" "$PAGE1" '.total' 100
assert_jq "Alice calls [get_coins] page 1 returns 25 coins" "$PAGE1" '.coins | length' "25"

PAGE2=$(rpc_a "get_coins" '{"offset": 25, "limit": 25, "sort_mode": "amount", "ascending": false}')
assert_jq "Alice calls [get_coins] page 2 returns 25 coins" "$PAGE2" '.coins | length' "25"

PAGE3=$(rpc_a "get_coins" '{"offset": 50, "limit": 25, "sort_mode": "amount", "ascending": false}')
assert_jq "Alice calls [get_coins] page 3 returns 25 coins" "$PAGE3" '.coins | length' "25"

PAGE4=$(rpc_a "get_coins" '{"offset": 75, "limit": 25, "sort_mode": "amount", "ascending": false}')
assert_jq_gte "Alice calls [get_coins] page 4 returns ≥1 coin" "$PAGE4" '.coins | length' 1

# Verify no overlap between pages
ALL_IDS=$(echo "$PAGE1 $PAGE2 $PAGE3 $PAGE4" | jq -s '[.[].coins[].coin_id] | unique | length')
TOTAL_FETCHED=$(echo "$PAGE1 $PAGE2 $PAGE3 $PAGE4" | jq -s '[.[].coins[]] | length')
if [[ "$ALL_IDS" == "$TOTAL_FETCHED" ]]; then
    echo "  ✅ Alice calls [get_coins] and returns unique coins across pages ($ALL_IDS unique)"
    PASSED=$((PASSED + 1))
else
    echo "  ❌ Alice calls [get_coins] but has duplicate coins across pages ($ALL_IDS unique out of $TOTAL_FETCHED)"
    FAILED=$((FAILED + 1))
fi

# Beyond-range page returns empty
BEYOND=$(rpc_a "get_coins" "$(jq -n --argjson t "$TOTAL_COINS" '{offset: $t, limit: 25, sort_mode: "amount", ascending: false}')")
assert_jq "Alice calls [get_coins] beyond-range page returns 0 coins" "$BEYOND" '.coins | length' "0"

log_done "✅ Alice split into $COIN_COUNT coins, pagination verified"

# ─── Alice sends 2M mojos to Bob + Alice creates DID ──────────────────────
log_start "💸 Alice sends 2M mojos to Bob + Alice creates DID (parallel)"

SEND_RESULT=$(rpc_a "send_xch" "$(jq -n --arg addr "$ADDR_B" '{
    address: $addr,
    amount: "2000000",
    fee: "100000",
    memos: [],
    auto_submit: true
}')")
assert_jq "Alice calls [send_xch] to send 2M mojos to Bob" "$SEND_RESULT" '.summary | length > 0' "true"

DID_RESULT=$(rpc_a "create_did" '{"name": "Smoke DID", "fee": "100000", "auto_submit": true}')
assert_jq "Alice calls [create_did] successfully" "$DID_RESULT" '.summary | length > 0' "true"

# Single wait — both Alice txns confirm in the same block
wait_for_pending_clear "$DIR_A" "$PORT_A" "Waiting for Alice's send + DID to confirm"

# Verify Bob received funds (brief sync after block confirmed)
wait_for_balance "$DIR_B" "$PORT_B" 1000000 "Waiting for Bob to receive 2M mojos" || die "Bob not funded"

# Verify Alice's DID
DIDS_A=$(rpc_a "get_dids" '{}')
assert_jq_gte "Alice calls [get_dids] and has ≥1 DID" "$DIDS_A" '.dids | length' 1
DID_ID=$(echo "$DIDS_A" | jq -r '.dids[0].launcher_id' 2>/dev/null) || DID_ID=""
info "DID_ID: $DID_ID"

log_done "✅ Bob funded, Alice has DID"

# ─── Bob issues CAT + Alice mints NFT under her DID ───────────────────────
log_start "🏗️  Bob issues CAT (1M mojos) + Alice mints NFT under her DID (parallel)"

CAT_RESULT=$(rpc_b "issue_cat" '{
    "name": "Smoke CAT",
    "ticker": "SMOK",
    "amount": "1000000",
    "fee": "100000",
    "auto_submit": true
}')
assert_jq "Bob calls [issue_cat] successfully" "$CAT_RESULT" '.summary | length > 0' "true"

# data_hash is the SHA-256 of https://sagewallet.net/assets/icon.png
# edition_total: 0 means unlimited edition
NFT_RESULT=$(rpc_a "bulk_mint_nfts" "$(jq -n --arg did "$DID_ID" '{
    mints: [{
        data_uris: ["https://sagewallet.net/assets/icon.png"],
        metadata_uris: ["https://sagewallet.net/assets/icon.json"],
        license_uris: ["https://sagewallet.net/assets/license.txt"],
        data_hash: "1b16adcd83e76b958b97d2b25db015909be7e83339c3275384fbe169d44cfa0b",
        edition_number: 1,
        edition_total: 0
    }],
    did_id: $did,
    fee: "100000",
    auto_submit: true
}')")
assert_jq "Alice calls [bulk_mint_nfts] successfully" "$NFT_RESULT" '.summary | length > 0' "true"

# Single wait — both confirm in the same block
wait_for_pending_clear "$DIR_A" "$PORT_A" "Waiting for Alice's NFT mint to confirm"
wait_for_pending_clear "$DIR_B" "$PORT_B" "Waiting for Bob's CAT issuance to confirm"

# Verify Bob's CAT
CATS_B=$(rpc_b "get_cats" '{}')
assert_jq_gte "Bob calls [get_cats] and has ≥1 CAT" "$CATS_B" '.cats | length' 1
ASSET_ID=$(echo "$CATS_B" | jq -r '.cats[0].asset_id' 2>/dev/null) || ASSET_ID=""
info "ASSET_ID: $ASSET_ID"

# Verify Alice's NFT
NFTS_A=$(rpc_a "get_nfts" '{"offset": 0, "limit": 10, "sort_mode": "name", "include_hidden": true}')
assert_jq_gte "Alice calls [get_nfts] and has ≥1 NFT" "$NFTS_A" '.nfts | length' 1
NFT_ID=$(echo "$NFTS_A" | jq -r '.nfts[0].launcher_id' 2>/dev/null) || NFT_ID=""
info "NFT_ID: $NFT_ID"

log_done "✅ Bob has CAT, Alice has NFT"

# ─── Alice accepts Bob's offer of 500k CAT for 50k mojos + Alice transfers NFT to Bob ───────────────────────
log_start "🤝 Alice accepts Bob's offer of 500k CAT for 50k mojos + Alice transfers NFT to Bob"

# Bob makes offer (no blockchain txn — just creates the offer string)
OFFER_RESULT=$(rpc_b "make_offer" "$(jq -n --arg asset "$ASSET_ID" '{
    offered_assets: [{
        asset_id: $asset,
        amount: "500000"
    }],
    requested_assets: [{
        asset_id: null,
        amount: "50000"
    }],
    fee: "100000",
    auto_import: true
}')")
OFFER_STR=$(echo "$OFFER_RESULT" | jq -r '.offer // empty' 2>/dev/null) || OFFER_STR=""
OFFER_ID=$(echo "$OFFER_RESULT" | jq -r '.offer_id // empty' 2>/dev/null) || OFFER_ID=""
assert_jq "Bob calls [make_offer] (500K CAT for 50K mojos)" "$OFFER_RESULT" '.offer | length > 0' "true"
info "Offer ID: $OFFER_ID"

# Alice takes the offer (on-chain)
TAKE_RESULT=$(rpc_a "take_offer" "$(jq -n --arg offer "$OFFER_STR" '{
    offer: $offer,
    fee: "100000",
    auto_submit: true
}')")
assert_jq "Alice calls [take_offer] successfully" "$TAKE_RESULT" '.summary | length > 0' "true"

# Alice transfers NFT to Bob (on-chain, same block)
TRANSFER_NFT_RESULT=$(rpc_a "transfer_nfts" "$(jq -n --arg nft "$NFT_ID" --arg addr "$ADDR_B" '{
    nft_ids: [$nft], address: $addr, fee: "100", auto_submit: true
}')")
assert_jq "Alice calls [transfer_nfts] to send NFT to Bob" "$TRANSFER_NFT_RESULT" '.summary | length > 0' "true"

# Single wait — both Alice txns confirm in the same block
wait_for_pending_clear "$DIR_A" "$PORT_A" "Waiting for Alice's offer + NFT transfer to confirm"

# Verify both wallets have CATs
CATS_A=$(rpc_a "get_cats" '{}')
assert_jq_gte "Alice calls [get_cats] and has ≥1 CAT from offer" "$CATS_A" '.cats | length' 1
CATS_B=$(rpc_b "get_cats" '{}')
assert_jq_gte "Bob calls [get_cats] and still has CATs" "$CATS_B" '.cats | length' 1

# Verify Bob received NFT (brief sync poll)
NFTS_B=$(wait_for_sync "get_nfts" '.nfts | length' "$DIR_B" "$PORT_B" "Bob received NFT" \
    '{"offset": 0, "limit": 10, "sort_mode": "name", "include_hidden": true}') || true
assert_jq_gte "Bob calls [get_nfts] and received NFT from Alice" "$NFTS_B" '.nfts | length' 1
info "Bob has NFT: $NFT_ID"

log_done "✅ Offer settled, Bob has NFT, Alice has CAT"

# ─── Alice transfers DID, Bob sends CAT + adds URI to NFT ────────────────
log_start "🔀 Alice transfers DID to Bob, Bob sends 100K CAT to Alice + adds URI to NFT"

TRANSFER_DID_RESULT=$(rpc_a "transfer_dids" "$(jq -n --arg did "$DID_ID" --arg addr "$ADDR_B" '{
    did_ids: [$did], address: $addr, fee: "100", auto_submit: true
}')")
assert_jq "Alice calls [transfer_dids] to send DID to Bob" "$TRANSFER_DID_RESULT" '.summary | length > 0' "true"

SEND_CAT_RESULT=$(rpc_b "send_cat" "$(jq -n --arg asset "$ASSET_ID" --arg addr "$ADDR_A" '{
    asset_id: $asset,
    address: $addr,
    amount: "100000",
    fee: "100000",
    memos: [],
    auto_submit: true
}')")
assert_jq "Bob calls [send_cat] to send 100K CAT to Alice" "$SEND_CAT_RESULT" '.summary | length > 0' "true"

ADD_URI_RESULT=$(rpc_b "add_nft_uri" "$(jq -n --arg nft "$NFT_ID" '{
    nft_id: $nft, uri: "https://sagewallet.net/assets/icon.png?added_uri=true", fee: "100", kind: "data", auto_submit: true
}')")
assert_jq "Bob calls [add_nft_uri] successfully" "$ADD_URI_RESULT" '.summary | length > 0' "true"

# Single wait — both wallets confirm in the same block
wait_for_pending_clear "$DIR_A" "$PORT_A" "Waiting for Alice's DID transfer to confirm"
wait_for_pending_clear "$DIR_B" "$PORT_B" "Waiting for Bob's send CAT + add URI to confirm"

# Verify Bob received DID (brief sync poll)
DIDS_B=$(wait_for_sync "get_dids" '.dids | length' "$DIR_B" "$PORT_B" "Bob received DID") || true
assert_jq_gte "Bob calls [get_dids] and received DID from Alice" "$DIDS_B" '.dids | length' 1

# If DID wasn't detected, dump sync logs for debugging
if [[ "$(echo "$DIDS_B" | jq '.dids | length' 2>/dev/null)" -lt 1 ]] 2>/dev/null; then
    log "--- Bob sync log (DID-related) ---"
    grep -i -E "did|singleton|custody|puzzle_hash|subscribe|unsynced|batch" "$DIR_B"/log/app.log* 2>/dev/null | tail -50 || true
    log "--- end Bob sync log ---"
fi

log_done "✅ Bob has DID + URI updated, Alice has CAT"

# ─── Alice combines coins + offers CAT for NFT back ──────────────────────
log_start "🔗 Alice combines coins, offers 25K CAT to get NFT back from Bob"

# Get all of Alice's coins and combine them
TOTAL=$(rpc_a "get_spendable_coin_count" '{}' | jq '.count' 2>/dev/null) || TOTAL=0
ALL_COINS=$(rpc_a "get_coins" "$(jq -n --argjson t "$TOTAL" '{offset: 0, limit: ([$t, 1] | max), sort_mode: "amount", ascending: false}')")
ALL_COIN_IDS=$(echo "$ALL_COINS" | jq '[.coins[].coin_id]')
COMBINE_RESULT=$(rpc_a "combine" "$(jq -n --argjson ids "$ALL_COIN_IDS" '{
    coin_ids: $ids, fee: "100", auto_submit: true
}')")
assert_jq "Alice calls [combine] on $TOTAL coins" "$COMBINE_RESULT" '.summary | length > 0' "true"

# Alice makes offer: 25K CAT for Bob's NFT (no fee — CAT coins are separate from the XCH combine)
OFFER_BACK_RESULT=$(rpc_a "make_offer" "$(jq -n --arg asset "$ASSET_ID" --arg nft "$NFT_ID" '{
    offered_assets: [{
        asset_id: $asset,
        amount: "25000"
    }],
    requested_assets: [{
        asset_id: $nft,
        amount: "1"
    }],
    fee: "0",
    auto_import: true
}')")
OFFER_BACK_STR=$(echo "$OFFER_BACK_RESULT" | jq -r '.offer // empty' 2>/dev/null) || OFFER_BACK_STR=""
assert_jq "Alice calls [make_offer] (25K CAT for her NFT back)" "$OFFER_BACK_RESULT" '.offer | length > 0' "true"

# Bob takes the offer (sends NFT, receives 25K CAT)
TAKE_BACK_RESULT=$(rpc_b "take_offer" "$(jq -n --arg offer "$OFFER_BACK_STR" '{
    offer: $offer,
    fee: "100000",
    auto_submit: true
}')")
assert_jq "Bob calls [take_offer] to accept Alice's offer" "$TAKE_BACK_RESULT" '.summary | length > 0' "true"

# Wait for combine + offer to settle
wait_for_pending_clear "$DIR_A" "$PORT_A" "Waiting for Alice's combine to confirm"
wait_for_pending_clear "$DIR_B" "$PORT_B" "Waiting for Bob's take_offer (NFT back) to confirm"

# Verify Alice has NFT back
NFTS_A=$(wait_for_sync "get_nfts" '.nfts | length' "$DIR_A" "$PORT_A" "Alice received NFT back" \
    '{"offset": 0, "limit": 10, "sort_mode": "name", "include_hidden": true}') || true
assert_jq_gte "Alice calls [get_nfts] and got NFT back from Bob" "$NFTS_A" '.nfts | length' 1

# Verify Alice combined down (may have a few coins from offer change)
AFTER_COMBINE=$(rpc_a "get_spendable_coin_count" '{}')
info "Alice has $(echo "$AFTER_COMBINE" | jq '.count') coins after combine + offer"

log_done "✅ Alice combined coins and got her NFT back"

# ═══════════════════════════════════════════════════════════════════════════
# NON-BLOCKCHAIN PHASES — read-only queries, metadata updates, final checks
# ═══════════════════════════════════════════════════════════════════════════

# ─── Read-only queries ─────────────────────────────────────────────────────
log_start "📖 Read-only queries (Alice + Bob in parallel)"

RESULT_A=$(mktemp "${TMPDIR:-/tmp}/sage-result-a.XXXXXX")
RESULT_B=$(mktemp "${TMPDIR:-/tmp}/sage-result-b.XXXXXX")
OUT_A=$(mktemp "${TMPDIR:-/tmp}/sage-out-a.XXXXXX")
OUT_B=$(mktemp "${TMPDIR:-/tmp}/sage-out-b.XXXXXX")

# --- Alice: key, address, network, coin, CAT queries ---
(
    P=0; F=0

    # Key management
    json=$(rpc_a "get_keys" '{}')
    check "Alice calls [get_keys] and returns ≥1 key" "$(echo "$json" | jq '(.keys | length) >= 1')"

    json=$(rpc_a "get_key" "$(jq -n --argjson fp "$FP_A" '{fingerprint: $fp}')")
    check "Alice calls [get_key] successfully" "$(echo "$json" | jq '.key != null')"

    json=$(rpc_a "get_secret_key" "$(jq -n --argjson fp "$FP_A" '{fingerprint: $fp}')")
    check "Alice calls [get_secret_key] and returns mnemonic" "$(echo "$json" | jq '(.secrets.mnemonic | length) > 0')"

    # Address validation
    json=$(rpc_a "check_address" "$(jq -n --arg addr "$ADDR_A" '{address: $addr}')")
    check "Alice calls [check_address] and validates own address" "$(echo "$json" | jq '.valid == true')"

    json=$(rpc_a "check_address" '{"address": "not-a-real-address"}')
    check "Alice calls [check_address] and rejects invalid address" "$(echo "$json" | jq '.valid == false')"

    # Derivations
    json=$(rpc_a "get_derivations" '{"hardened": false, "offset": 0, "limit": 5}')
    check "Alice calls [get_derivations] and returns ≥1" "$(echo "$json" | jq '(.derivations | length) >= 1')"

    # Network
    json=$(rpc_a "get_networks" '{}')
    check "Alice calls [get_networks] successfully" "$(echo "$json" | jq 'type == "object"')"

    json=$(rpc_a "get_network" '{}')
    check "Alice calls [get_network] and shows testnet" "$(echo "$json" | jq '.kind == "testnet"')"

    # Database
    json=$(rpc_a "get_database_stats" '{}')
    check "Alice calls [get_database_stats] successfully" "$(echo "$json" | jq '.total_pages > 0')"

    # Coins
    json=$(rpc_a "get_coins" '{"offset": 0, "limit": 10, "sort_mode": "created_height", "ascending": false}')
    check "Alice calls [get_coins] and returns ≥1 coin" "$(echo "$json" | jq '(.coins | length) >= 1')"

    json=$(rpc_a "get_spendable_coin_count" '{}')
    check "Alice calls [get_spendable_coin_count] and returns ≥1" "$(echo "$json" | jq '.count >= 1')"

    # CATs (Alice has CATs from offer)
    json=$(rpc_a "get_cats" '{}')
    check "Alice calls [get_cats] and returns ≥1" "$(echo "$json" | jq '(.cats | length) >= 1')"

    # Transactions
    txn_json=$(rpc_a "get_transactions" '{"offset": 0, "limit": 1, "ascending": false}')
    check "Alice calls [get_transactions] and returns ≥1" "$(echo "$txn_json" | jq '.total >= 1')"

    height=$(echo "$txn_json" | jq -r '.transactions[0].height // "0"' 2>/dev/null) || height="0"
    if [[ "$height" != "0" && "$height" != "null" ]]; then
        json=$(rpc_a "get_transaction" "$(jq -n --argjson h "$height" '{height: $h}')")
        check "Alice calls [get_transaction] successfully" "$(echo "$json" | jq '.transaction != null')"
    else
        echo "  ❌ Alice calls [get_transaction] (no height available)"; F=$((F + 1))
    fi

    # Database maintenance
    json=$(rpc_a "perform_database_maintenance" '{"force_vacuum": false}')
    check_ok "Alice calls [perform_database_maintenance] successfully" "$json"

    # NFTs (Alice got her NFT back in step 9)
    json=$(rpc_a "get_nfts" '{"offset": 0, "limit": 10, "sort_mode": "name", "include_hidden": true}')
    check "Alice calls [get_nfts] and returns ≥1 NFT" "$(echo "$json" | jq '(.nfts | length) >= 1')"

    json=$(rpc_a "get_nft" "$(jq -n --arg id "$NFT_ID" '{nft_id: $id}')")
    check "Alice calls [get_nft] and returns specific NFT" "$(echo "$json" | jq '.nft != null')"

    json=$(rpc_a "get_nft_collections" '{"offset": 0, "limit": 10, "include_hidden": true}')
    check_ok "Alice calls [get_nft_collections] successfully" "$json"

    json=$(rpc_a "get_nft_collection" '{"collection_id": null}')
    check_ok "Alice calls [get_nft_collection] successfully" "$json"

    # Options
    json=$(rpc_a "get_options" '{"offset": 0, "limit": 10}')
    check "Alice calls [get_options] and returns list" "$(echo "$json" | jq 'has("options")')"

    echo "$P $F" > "$RESULT_A"
) > "$OUT_A" 2>&1 &
PID_READ_A=$!

# --- Bob: key, CAT, NFT, DID, coin queries ---
(
    P=0; F=0

    # Key management
    json=$(rpc_b "get_keys" '{}')
    check "Bob calls [get_keys] and returns ≥1 key" "$(echo "$json" | jq '(.keys | length) >= 1')"

    json=$(rpc_b "get_key" "$(jq -n --argjson fp "$FP_B" '{fingerprint: $fp}')")
    check "Bob calls [get_key] successfully" "$(echo "$json" | jq '.key != null')"

    # Address validation
    json=$(rpc_b "check_address" "$(jq -n --arg addr "$ADDR_B" '{address: $addr}')")
    check "Bob calls [check_address] and validates own address" "$(echo "$json" | jq '.valid == true')"

    json=$(rpc_b "check_address" '{"address": "xyz123invalid"}')
    check "Bob calls [check_address] and rejects invalid address" "$(echo "$json" | jq '.valid == false')"

    # CAT/token queries
    json=$(rpc_b "get_all_cats" '{}')
    check "Bob calls [get_all_cats] and returns ≥1" "$(echo "$json" | jq '(.cats | length) >= 1')"

    json=$(rpc_b "get_token" "$(jq -n --arg id "$ASSET_ID" '{asset_id: $id}')")
    check "Bob calls [get_token] and returns CAT info" "$(echo "$json" | jq '.token != null')"

    # Coins
    json=$(rpc_b "get_coins" '{"offset": 0, "limit": 10, "sort_mode": "created_height", "ascending": false}')
    check "Bob calls [get_coins] and returns ≥1 coin" "$(echo "$json" | jq '(.coins | length) >= 1')"

    json=$(rpc_b "get_spendable_coin_count" '{}')
    check "Bob calls [get_spendable_coin_count] and returns ≥1" "$(echo "$json" | jq '.count >= 1')"

    json=$(rpc_b "get_spendable_coin_count" "$(jq -n --arg id "$ASSET_ID" '{asset_id: $id}')")
    check "Bob calls [get_spendable_coin_count] (CAT) and returns ≥1" "$(echo "$json" | jq '.count >= 1')"

    # Asset ownership
    json=$(rpc_b "is_asset_owned" "$(jq -n --arg id "$ASSET_ID" '{asset_id: $id}')")
    check "Bob calls [is_asset_owned] (CAT) and returns true" "$(echo "$json" | jq '.owned == true')"

    json=$(rpc_b "is_asset_owned" "$(jq -n --arg id "$DID_ID" '{asset_id: $id}')")
    check "Bob calls [is_asset_owned] (DID) and returns true" "$(echo "$json" | jq '.owned == true')"

    # Coin-specific queries
    coins_json=$(rpc_b "get_coins" '{"offset": 0, "limit": 1, "sort_mode": "created_height", "ascending": false}')
    coin_id=$(echo "$coins_json" | jq -r '.coins[0].coin_id // ""' 2>/dev/null) || coin_id=""
    if [[ -n "$coin_id" && "$coin_id" != "null" ]]; then
        json=$(rpc_b "get_coins_by_ids" "$(jq -n --arg id "$coin_id" '{coin_ids: [$id]}')")
        check "Bob calls [get_coins_by_ids] and returns coin" "$(echo "$json" | jq '(.coins | length) >= 1')"

        json=$(rpc_b "get_are_coins_spendable" "$(jq -n --arg id "$coin_id" '{coin_ids: [$id]}')")
        check "Bob calls [get_are_coins_spendable] and returns true" "$(echo "$json" | jq '.spendable == true')"
    else
        echo "  ❌ Bob calls [get_coins_by_ids] (no coin_id)"; F=$((F + 1))
        echo "  ❌ Bob calls [get_are_coins_spendable] (no coin_id)"; F=$((F + 1))
    fi

    # NFT queries (Alice got the NFT back, Bob has none)
    json=$(rpc_b "get_nfts" '{"offset": 0, "limit": 10, "sort_mode": "name", "include_hidden": true}')
    check "Bob calls [get_nfts] and returns 0 NFTs" "$(echo "$json" | jq '(.nfts | length) == 0')"

    json=$(rpc_b "get_nft_collections" '{"offset": 0, "limit": 10, "include_hidden": true}')
    check_ok "Bob calls [get_nft_collections] successfully" "$json"

    # DID queries (Bob owns the DID now)
    json=$(rpc_b "get_dids" '{}')
    check "Bob calls [get_dids] and returns ≥1" "$(echo "$json" | jq '(.dids | length) >= 1')"

    json=$(rpc_b "get_minter_did_ids" '{"offset": 0, "limit": 10}')
    check_ok "Bob calls [get_minter_did_ids] successfully" "$json"

    # Themes
    json=$(rpc_b "get_user_themes" '{}')
    check_ok "Bob calls [get_user_themes] successfully" "$json"

    echo "$P $F" > "$RESULT_B"
) > "$OUT_B" 2>&1 &
PID_READ_B=$!

collect_parallel "$PID_READ_A" "$PID_READ_B" "$RESULT_A" "$RESULT_B" "$OUT_A" "$OUT_B"

log_done "✅ Read-only queries done"

# ─── Non-blockchain operations ─────────────────────────────────────────────
log_start "🔧 Non-blockchain operations (Alice + Bob in parallel)"

RESULT_A=$(mktemp "${TMPDIR:-/tmp}/sage-result-a.XXXXXX")
RESULT_B=$(mktemp "${TMPDIR:-/tmp}/sage-result-b.XXXXXX")
OUT_A=$(mktemp "${TMPDIR:-/tmp}/sage-out-a.XXXXXX")
OUT_B=$(mktemp "${TMPDIR:-/tmp}/sage-out-b.XXXXXX")

# --- Alice: view_offer, rename_key, increase_derivation_index ---
(
    P=0; F=0

    # View offer (parses offer string without importing)
    json=$(rpc_a "view_offer" "$(jq -n --arg o "$OFFER_STR" '{offer: $o}')")
    check "Alice calls [view_offer] and parses offer" "$(echo "$json" | jq '.offer != null')"

    # Rename key
    rpc_a "rename_key" "$(jq -n --argjson fp "$FP_A" '{fingerprint: $fp, name: "Alice Renamed"}')" >/dev/null 2>&1
    json=$(rpc_a "get_key" "$(jq -n --argjson fp "$FP_A" '{fingerprint: $fp}')")
    check "Alice calls [rename_key] and updates name" "$(echo "$json" | jq '.key.name == "Alice Renamed"')"

    # Increase derivation index
    json=$(rpc_a "increase_derivation_index" '{"index": 10}')
    check_ok "Alice calls [increase_derivation_index] successfully" "$json"

    # Toggle NFT visibility (Alice owns the NFT)
    json=$(rpc_a "update_nft" "$(jq -n --arg id "$NFT_ID" '{nft_id: $id, visible: false}')")
    check_ok "Alice calls [update_nft] (hide) successfully" "$json"

    json=$(rpc_a "update_nft" "$(jq -n --arg id "$NFT_ID" '{nft_id: $id, visible: true}')")
    check_ok "Alice calls [update_nft] (show) successfully" "$json"

    # Redownload NFT data
    json=$(rpc_a "redownload_nft" "$(jq -n --arg id "$NFT_ID" '{nft_id: $id}')")
    check_ok "Alice calls [redownload_nft] successfully" "$json"

    echo "$P $F" > "$RESULT_A"
) > "$OUT_A" 2>&1 &
PID_OPS_A=$!

# --- Bob: offer queries, emoji, delete/import offer, rename, settings, metadata ---
(
    P=0; F=0

    # Offer record queries
    json=$(rpc_b "get_offers" '{}')
    check "Bob calls [get_offers] and returns ≥1 offer" "$(echo "$json" | jq '(.offers | length) >= 1')"

    json=$(rpc_b "get_offer" "$(jq -n --arg id "$OFFER_ID" '{offer_id: $id}')")
    check "Bob calls [get_offer] and returns specific offer" "$(echo "$json" | jq '.offer != null')"

    json=$(rpc_b "get_offers_for_asset" "$(jq -n --arg id "$ASSET_ID" '{asset_id: $id}')")
    check_ok "Bob calls [get_offers_for_asset] successfully" "$json"

    # Set wallet emoji
    rpc_b "set_wallet_emoji" "$(jq -n --argjson fp "$FP_B" '{fingerprint: $fp, emoji: "T"}')" >/dev/null 2>&1
    json=$(rpc_b "get_key" "$(jq -n --argjson fp "$FP_B" '{fingerprint: $fp}')")
    check "Bob calls [set_wallet_emoji] and updates emoji" "$(echo "$json" | jq '.key.emoji == "T"')"

    # Delete offer, then re-import (tests both endpoints)
    json=$(rpc_b "delete_offer" "$(jq -n --arg id "$OFFER_ID" '{offer_id: $id}')")
    check_ok "Bob calls [delete_offer] successfully" "$json"

    json=$(rpc_b "import_offer" "$(jq -n --arg o "$OFFER_STR" '{offer: $o}')")
    check "Bob calls [import_offer] and returns offer_id" "$(echo "$json" | jq '(.offer_id | length) > 0')"

    # Rename key
    rpc_b "rename_key" "$(jq -n --argjson fp "$FP_B" '{fingerprint: $fp, name: "Bob Renamed"}')" >/dev/null 2>&1
    json=$(rpc_b "get_key" "$(jq -n --argjson fp "$FP_B" '{fingerprint: $fp}')")
    check "Bob calls [rename_key] and updates name" "$(echo "$json" | jq '.key.name == "Bob Renamed"')"

    # Peer settings (toggle and restore)
    json=$(rpc_b "set_discover_peers" '{"discover_peers": false}')
    check_ok "Bob calls [set_discover_peers] (off) successfully" "$json"
    json=$(rpc_b "set_discover_peers" '{"discover_peers": true}')
    check_ok "Bob calls [set_discover_peers] (on) successfully" "$json"

    json=$(rpc_b "set_target_peers" '{"target_peers": 3}')
    check_ok "Bob calls [set_target_peers] (3) successfully" "$json"
    json=$(rpc_b "set_target_peers" '{"target_peers": 5}')
    check_ok "Bob calls [set_target_peers] (5) successfully" "$json"

    # Update CAT metadata
    token_json=$(rpc_b "get_token" "$(jq -n --arg id "$ASSET_ID" '{asset_id: $id}')")
    json=$(rpc_b "update_cat" "$(echo "$token_json" | jq '{record: (.token + {name: "Smoke CAT Updated"})}')")
    check_ok "Bob calls [update_cat] successfully" "$json"

    # Update DID metadata (Bob owns the DID now)
    json=$(rpc_b "update_did" "$(jq -n --arg id "$DID_ID" '{did_id: $id, name: "Smoke DID Updated", visible: true}')")
    check_ok "Bob calls [update_did] successfully" "$json"

    echo "$P $F" > "$RESULT_B"
) > "$OUT_B" 2>&1 &
PID_OPS_B=$!

collect_parallel "$PID_OPS_A" "$PID_OPS_B" "$RESULT_A" "$RESULT_B" "$OUT_A" "$OUT_B"

log_done "✅ Non-blockchain operations done"

# ─── Final state check ─────────────────────────────────────────────────────
log_start "🔍 Final state check"

# Transaction history on both wallets
TXN_A=$(rpc_a "get_transactions" '{"offset": 0, "limit": 10, "ascending": false}')
assert_jq_gte "Alice calls [get_transactions] and has ≥1 transaction" "$TXN_A" '.total' 1

TXN_B=$(rpc_b "get_transactions" '{"offset": 0, "limit": 10, "ascending": false}')
assert_jq_gte "Bob calls [get_transactions] and has ≥1 transaction" "$TXN_B" '.total' 1

# Peer connectivity
PEERS_A=$(rpc_a "get_peers" '{}')
PEER_COUNT_A=$(echo "$PEERS_A" | jq '.peers | length' 2>/dev/null) || PEER_COUNT_A=0
info "Alice peer count: $PEER_COUNT_A"

PEERS_B=$(rpc_b "get_peers" '{}')
PEER_COUNT_B=$(echo "$PEERS_B" | jq '.peers | length' 2>/dev/null) || PEER_COUNT_B=0
info "Bob peer count: $PEER_COUNT_B"

# Logout and re-login on Bob (tests auth cycle)
json=$(rpc_b "logout" '{}')
assert_jq "Bob calls [logout] successfully" "$json" 'type' "object"

json=$(rpc_b "login" "$(jq -n --argjson fp "$FP_B" '{fingerprint: $fp}')")
assert_jq "Bob calls [login] successfully" "$json" 'type' "object"

# Give Bob a moment to re-sync after login
sleep 3

log_done "✅ Final state check done"

# ─── Return funds ──────────────────────────────────────────────────────────
if [[ -n "$RETURN_ADDRESS" ]]; then
    log_start "💸 Returning remaining funds to $RETURN_ADDRESS"

    return_wallet_funds rpc_b "Bob"
    return_wallet_funds rpc_a "Alice"

    # Wait for both to clear
    wait_for_pending_clear "$DIR_B" "$PORT_B" "Waiting for Bob's return to confirm"
    wait_for_pending_clear "$DIR_A" "$PORT_A" "Waiting for Alice's return to confirm"

    log_done "✅ Funds returned (may take a block to confirm)"
    echo ""
else
    echo ""
    echo "  💡 TIP: To recover remaining funds, re-run with --return-address <your-txch-address>"
    echo "  Alice mnemonic: $MNEMONIC_A"
    echo "  Bob mnemonic: $MNEMONIC_B"
    echo ""
fi

# ─── Summary ────────────────────────────────────────────────────────────────
echo ""
echo "  🔐 Test wallet seed phrases (for manual inspection):"
echo "    Alice: $MNEMONIC_A"
echo "    Bob: $MNEMONIC_B"
echo ""
if [[ $FAILED -gt 0 ]]; then
    echo -e "${RED}  ❌ Some smoke tests failed!${RESET}"
    exit 1
else
    echo -e "${GREEN}  🎉 All smoke tests passed!${RESET}"
    exit 0
fi
