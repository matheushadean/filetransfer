use anyhow::Result;
use clap::{ArgGroup, CommandFactory, Parser};
use std::{
    fs::File,
    net::{TcpListener, TcpStream},
    path::Path,
    process::exit,
};

#[derive(Parser, Debug)]
#[clap(group(
    ArgGroup::new("serve")
        .multiple(true)
        .args(&["mappings", "serve_address"])
        .requires_all(&["mappings", "serve_address"])
))]
struct ServeArgs {
    /// files to be mapped inside the tarball
    #[arg(long, short)]
    #[clap(requires = "serve_address", default_values_t = Vec::<String>::new())]
    mappings: Vec<String>,

    /// Address to serve to (addr:port)
    #[arg(long, short)]
    #[clap(requires = "mappings", default_value_t = String::new())]
    serve_address: String,
}

#[derive(Parser, Debug)]
#[clap(group(
    ArgGroup::new("listen")
        .multiple(true)
        .conflicts_with_all(&["serve"])
        .args(&["listen_port", "listen_destination"])
        .requires_all(&["listen_port", "listen_destination"])
))]
struct ListenArgs {
    /// listen to TCP connections
    #[arg(long, short = 'p', default_value_t = 0)]
    #[clap(requires = "listen_destination")]
    listen_port: u16,

    /// Directory where files from TCP listener are saved to
    #[arg(long, short = 'd', default_value_t = String::new())]
    #[clap(requires = "listen_port")]
    listen_destination: String,
}

// Transfer a rolling-checksummed tarball of
// given paths
#[derive(Parser, Debug)]
#[command(version = "0.0.1", about, long_about = None)]
struct ProgramArgs {
    #[clap(flatten)]
    serve_args: ServeArgs,

    #[clap(flatten)]
    listen_args: ListenArgs,
}

fn create_tarball<T: AsRef<Path>>(paths: &[T], destination: TcpStream) -> Result<()> {
    // Get the file info here, we need path
    // data, permisions
    let mut tar = tar::Builder::new(destination);
    for path in paths.iter().map(|p| p.as_ref()) {
        // TODO : need relative paths for the tar, it seems
        //        this library doesn't resolve the rel paths
        //        from the absolute ones
        if path.is_absolute() {
            anyhow::bail!("absolute mappings aren't yet supported");
        }

        if path.is_symlink() {
            anyhow::bail!("symlinks aren't supported");
        }

        if path.is_dir() {
            tar.append_dir_all(".", path)?;
            continue;
        }

        if path.is_file() {
            // Need to add more context to error here,
            // as Rust only returns "OS: file not found"
            let mut file = File::open(path);
            let file = match file.as_mut() {
                Ok(file) => Ok(file),
                Err(err) => Err(anyhow::anyhow!("{}: {}", path.display(), err)),
            }?;

            tar.append_file(path, file)?;
            continue;
        }
    }

    tar.finish()?;
    Ok(())
}

fn try_main(program_args: ProgramArgs) -> Result<()> {
    // If we are in the "serve" mode
    // Create the destination TcpStream
    if !program_args.serve_args.serve_address.is_empty() {
        // TODO : Create the TCP stream server
        let stream = TcpStream::connect(program_args.serve_args.serve_address)?;
        stream.set_nonblocking(true)?;
        create_tarball(&program_args.serve_args.mappings, stream)?;
        return Ok(());
    }

    // If we are in the "listen" mode
    if program_args.listen_args.listen_port != 0 {
        // Listen and then save to file
        let bind_addr = format!("0.0.0.0:{}", program_args.listen_args.listen_port);
        let listener = TcpListener::bind(bind_addr).unwrap();

        println!("listening to connection...");
        if let Ok((socket, addr)) = listener.accept() {
            println!("new client: {addr:?}");
            let mut archive = tar::Archive::new(socket);
            archive.unpack(program_args.listen_args.listen_destination)?;
            return Ok(());
        }

        anyhow::bail!("received no connections after a timeout");
    }

    anyhow::bail!("invalid mode")
}

fn main() {
    let program_args = ProgramArgs::parse();

    if program_args.listen_args.listen_destination.is_empty()
        && program_args.serve_args.serve_address.is_empty()
    {
        println!("Invalid calling mode");
        ProgramArgs::command()
            .print_long_help()
            .expect("could not print help");
        exit(-1);
    }

    match try_main(program_args) {
        Ok(()) => {}
        Err(e) => {
            println!("Error: {}", e);
            exit(-1);
        }
    }

    println!("finished");
}
