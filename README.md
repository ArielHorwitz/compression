# FFT Compression

This project is a proof of concept for media compression using the FFT algorithm.

It compresses and decompresses a *`.wav`* or *`.bmp`* file by applying an FFT algorithm and truncating higher frequencies.
It can also analyze a given media file by plotting both time/color and frequency domains.

```
$ cargo run -- --help
Usage: compression [OPTIONS] --file <FILE>

Options:
  -f, --file <FILE>                Input file (.wav or .bmp)
  -c, --compression <COMPRESSION>  Compression level (higher: smaller file size, lower: better quality) [default: 10]
  -a, --analyze                    Analyze frequencies
  -l, --log-factor <LOG_FACTOR>    Log factor (when analyzing) [default: 5]
  -o, --output-dir <OUTPUT_DIR>    Output directory [default: data]
  -h, --help                       Print help
  -V, --version                    Print version
```
