# Fuzzer for fontdue

## Usage

1. Build the fuzz target: `cargo afl build --release`

2. Generate a corpus directory. Ideally this would be some small font file(s). The last 4 bytes are parsed as utf8, and the last character is used as the rendered character -- so be sure to concatenate that onto the end of your file. `build_corpus.py` may be helpful to run.

3. `cargo afl fuzz -i CORPUS -o out target/release/fontdue-fuzz-target` (replace CORPUS with your corpus directory).

4. Wait (a while)

## Replicating a crash

Included is a convenience binary which accepts the crash file as an argument, reads it, and rasterizes the target character.

Invoke it with `./target/release/replicator out/crashes/INTERESTING_CRASH`
