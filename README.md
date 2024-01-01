# unique

Read lines from FILE (or standard input) with filters applied.

Repeated lines will be removed according to the chosen filters.

Writing the result to OUTPUT (or standard output).

Type in the terminal `unique -h` to see all available options:

```
Read lines from FILE (or standard input) removing any repeated lines according to the Options.

Usage: unique [OPTIONS] [FILE]

Arguments:
  [FILE]
          FILE input (or standard input)

Options:
  -e, --empty_lines
          Remove empty lines
  -i, --ignore_case
          Ignore differences in case when comparing lines
  -t, --trim
          Returns lines with leading and trailing whitespace removed
  -w, --whitespace
          Replace multiple whitespace with just one
  -c, --csv
          Parse valid CSV file.
          All lines must have the same number of columns.
          Columns are identified according to the delimiter character.
          The default delimiter character is ';'.
  -s, --separator <SEPARATOR>
          Set the field separator (delimiter) for CSV files to:
          comma ','
          semicolon ';'
          pipe '|'
          or another char. [default: ;]
  -d, --format_date
          Formats the date in %d/%m/%Y format in CSV files.
          Example:
          "1 / 1 / 2023" => "01/01/2023"
  -k, --format_key
          Format 44-digit key and calculate key check digit in CSV files.
  -n, --format_number
          Format numbers to float64 in CSV files.
          Example:
          34.542.675,01 => 34542675.01
          34,542,675.01 => 34542675.01
  -r, --repeated
          Print only duplicate or repeated lines
  -v, --verbose
          Show number of unique, repeated and total lines
  -h, --help
          Print help
  -V, --version
          Print version
```
## Building

You can build the program as follows:
```
git clone git@github.com:claudiofsr/unique.git
cd unique
cargo build --release
```
To install the program in "$HOME/.cargo/bin":
```
cargo install --path=.
```
Then run the program.

Examples:
```
unique input.csv -eitwcv > output.csv

echo -e "Test 1\n\na\na\nfinal\nFinal \ntest  1" | unique -eitw
```