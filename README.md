# FFT Compression

This project is a practice project for learning Rust and meant for educational purposes.

It compresses and decompresses a raw *`.wav`* file by applying an FFT algorithm and truncating higher frequencies.
It can also analyze a given media file by plotting both time and frequency domains.

```
$ cargo run -- --help
Usage: compression [OPTIONS] --file <FILE>

Options:
  -f, --file <FILE>                WAV file
  -a, --analyze                    Analyze frequencies
  -c, --freq-cutoff <FREQ_CUTOFF>  Frequency cutoff (when compressing) [default: 3000]
  -o, --output-dir <OUTPUT_DIR>    Output folder [default: ./]
  -h, --help                       Print help
  -V, --version                    Print version
```
