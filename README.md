# FFT Compression

Proof of concept for audio and image file compression and analysis using a custom implementation of the FFT algorithm.

![Analysis output](/analysis.png)

```
$ cargo run -- --help
Usage: compression [OPTIONS] <FILE>

Arguments:
  <FILE>  Input file (.wav or .bmp)

Options:
  -c, --compression <COMPRESSION>  Compression level (higher: smaller file size, lower: better quality) [default: 10]
  -a, --analyze                    Analyze frequencies
  -l, --log-factor <LOG_FACTOR>    Log factor (when analyzing) [default: 2.5]
  -o, --output-dir <OUTPUT_DIR>    Output directory [default: data]
  -h, --help                       Print help
  -V, --version                    Print version
```
