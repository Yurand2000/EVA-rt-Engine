#!/bin/sh

# check if therminal supports colored text
if command -v tput >/dev/null 2>&1 && [ $(tput colors) -gt 0 ]; then
    export TERM_COLORS=1
fi

run_test() {
    if [ $TERM_COLORS -eq 1 ]; then
        echo -en "Running \033[1m$1\033[0m, sched on config \033[1m$2\033[0m: "
    else
        echo -n "Running $1, sched on config $CONFIG_FILE: "
    fi

    run_test_generic $1 $2 0
}

run_test_fail() {
    if [ $TERM_COLORS -eq 1 ]; then
        echo -en "Running \033[1m$1\033[0m, \033[1mFAIL\033[0m sched on config \033[1m$2\033[0m: "
    else
        echo -n "Running $1, FAIL sched on config $2: "
    fi

    run_test_generic $1 $2 1
}

run_test_generic () {
    ./target/debug/eva-engine-cli -q -i "examples/$TESTDIR/$1" -c "examples/$TESTDIR/$2" 2> /tmp/out
    EXIT_CODE=$?
    if [ $EXIT_CODE -eq 2 ]; then
        if [ $TERM_COLORS -eq 1 ]; then
            echo -e "\033[33mParse/Data Error ✖\033[0m"
            echo -n "    "
            cat /tmp/out
        else
            echo "Parse/Data Error ✖"
            echo -n "    "
            cat /tmp/out
        fi
    elif [ $EXIT_CODE -eq $3 ]; then
        if [ $TERM_COLORS -eq 1 ]; then
            echo -e "\033[32mSuccess ✔\033[0m"
        else
            echo "Success ✔"
        fi
    else
        if [ $TERM_COLORS -eq 1 ]; then
            echo -e "\033[31mFailure ✖\033[0m"
        else
            echo "Failure ✖"
        fi
    fi
}

# Build the analyzer
echo "Building..."
cargo build

# Run Tests --------------------------------------------------------------------
echo "Running Examples..."

# UniProcessor Rate Monotonic
TESTDIR="up_rate_monotonic"
echo "- UniProcessor Rate Monotonic (examples/$TESTDIR)"
run_test taskset00.txt config_classic.json
run_test taskset00.txt config_simple.json
run_test taskset00.txt config_hyperbolic.json
run_test_fail taskset01.txt config_classic.json
run_test_fail taskset01.txt config_hyperbolic.json