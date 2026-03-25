# unique

A command-line tool to process lines, remove duplicates, and format data with CSV processing capabilities.

## Usage

 unique -h

``` 
Read lines from FILE (or standard input) removing any repeated lines according to the Options.

Usage: unique [OPTIONS] [FILE]

Arguments:
  [FILE]
          FILE input (or standard input if empty)

Options:
  -e, --remove-empty-lines
          Remove empty lines
  -i, --ignore-case
          Ignore differences in case when comparing lines
  -t, --trim-line
          Returns lines with leading and trailing whitespace removed
  -w, --replace-multiple-whitespaces
          Replace multiple whitespace with just one
  -c, --csv
          Parse valid CSV file.
          All lines must have the same number of columns based on the separator.
  -s, --separator <SEPARATOR>
          Set the field separator (delimiter) for CSV files to: comma ',' semicolon ';' pipe '|' or another char [default: ;]
  -d, --format_date
          Formats the date in %d/%m/%Y format in CSV files. Example: "1 / 1 / 2023" => "01/01/2023"
  -k, --format-key
          Format 44-digit NFe key and validates check digit in CSV files
  -f, --format-number
          Format numeric strings to computational float (f64) in CSV files.
  -n, --number-format <NUMBER_FORMAT>
          Choose the number format for parsing decimals and thousands. [default: brazilian] [possible values: brazilian, international]
  -r, --repeated
          Print only duplicate or repeated lines
  -v, --verbose
          Show number of unique, repeated and total lines
  -h, --help
          Print help (see more with '--help')
  -V, --version
          Print version
``` 

## Building

Build and install from source:
```
git clone git@github.com:claudiofsr/unique.git
cd unique
cargo b -r && cargo install --path=.
```

Then run the program.

## Didactic Examples

### 1. Basic Text Deduplication
#### Remove duplicates while ignoring case and extra spaces:
echo -e "apple\nApple\n  apple  " | unique -i -t
```
apple
```

### 2. Cleaning up a Text File
#### Remove empty lines, trim whitespace, and normalize spaces:
unique input.txt -e -t -w > cleaned_output.txt

### 3. CSV Processing (Brazilian Standard)
#### Input row: 352301...; 01 / 05 / 2023 ; 1.250,50
#### Command: Parse CSV, format NFe keys, dates, and numbers (BR pattern)
unique data.csv --csv --format-key --format-date --format-number --verbose

### 4. CSV Processing (International Standard)
#### Input row: "John Doe", "1,500.75", "2023-01-01"
#### Command: Use comma separator and international number formatting
unique data.csv -c -s ',' -f -n international

### 5. Finding Duplicates Only
#### Useful for log analysis to see which entries are repeating:
cat access.log | unique --repeated

### 6. Complex Pipeline
#### Filter specific errors from a log, then get unique entries with statistics:
grep "ERROR 500" server.log | unique -v

## Statistics Example (--verbose)
Running with -v will output the following to stderr:

echo -e "apple\nApple\n\n\n\n  apple  " | unique -vi -t

```
apple

Number of lines in the original file: 6
Number of unique lines              : 2
Number of repeated lines            : 4
Number of empty lines               : 3
Number of lines in the final file   : 2

Total Run Time: "0.002913 second (2.912845ms)"
```
