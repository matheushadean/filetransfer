# Filetransfer 📁

`filetransfer` is a Rust app for sending file differentials over a TCP stream.
It uses a rolling checksum algorithm along with the `tar` crate to efficiently
transmit file changes.

## Usage 🖥️

This app can be used to either serve files to a destination,
or to listen for tar data.

### Serving Files 💁

To run the serve mode, use the following command:

```sh
./filetransfer -m file1 -m file2 -s "0.0.0.0:5555"
```

The above command will serve files `file1` and `file2` as
a single tar archive to the address `0.0.0.0:5555`.

### Listening for Files 👂

To run the listen mode, use the following command:

```sh
./filetransfer -p 5555 -d dest_dir
```

The above command will listen for incomming TCP connections
in port `5555`, and will save the unpacked archives to
the `dest_dir` directory.

**Note** that the directory
needs to exist as the app will not create it.

## Features 📋

- [ ] Calculates file differences given rolling checksums.
- [x] Build tar archives from given files.
- [x] Send and receive file diffs over TCP streams.

## Dependencies

Dependencies worth mentioning

- `clap` to handle the generation for help text and cmd-line args.
- `tar` to build and unpack archives.
- `TODO` for checksums ¬¬

