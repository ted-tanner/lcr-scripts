# LCR Calling Diagram Generator

Simple program to generate a draw.io diagram from a dataset collected from LCR

## Running the program

The program must be run from the command-line. Usage:

``` shell
callings-diagram <input file> <output file>
```

The input file should contain the JSON response from [https://lcr.churchofjesuschrist.org/api/orgs/sub-orgs-with-callings?ip=true&lang=eng](https://lcr.churchofjesuschrist.org/api/orgs/sub-orgs-with-callings?ip=true&lang=eng). To obtain the JSON response, you must be [signed into LCR](https://lcr.churchofjesuschrist.org/). Navigate to the URL above in your browser after signing in and copy the text that appears and save it to the input file. `.json` is the recommended file extension for the input file.

The generated diagram will be written to the output file. The file will be overwritten (or created if it does not exist). `.drawio` is the recommended file extension for the output file.

The `diagram-config.json` file allows you to configure the appearance of the generated diagram. It *must* be in the current working directory of the shell you run the program from.

## Viewing or editing the diagram

The generated diagram can be uploaded to a file storage service (e.g. Google Drive) and viewed/edited on [draw.io](https://draw.io). There is also a draw.io desktop app available that allows you to view and edit the file without uploading it to a service.

## Building the binary (may require a little understanding of programming)

The program is written in Rust. You may [download the Rust compiler here](https://www.rust-lang.org/tools/install). Once installed and added to your path, build the binary by running the following command in the directory above the `src` folder:

``` shell
cargo build --release
```

The compiled binary should be in the `./target/release` directory.

