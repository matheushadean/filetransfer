use std::{fs::File, path::Path, process::exit};
use clap::Parser;
use anyhow::Result;

// Transfer a rolling-checksummed tarball of
// given paths
#[derive(Parser, Debug)]
#[command(version = "0.0.1", about, long_about = None)]
struct ProgramArgs {
    #[arg(long, short)]
    // files to be mapped inside the tarball
    mappings: Vec<String>,

    #[arg(long, short)]
    // path to the destination tar file
    dest: String,
}

fn create_tarball<T: AsRef<Path>>(paths: &[T], dest_path: T) -> Result<()> {
    // Get the file info here, we need path
    // data, permisions
    let out_file = File::create(dest_path)?;
    let mut tar = tar::Builder::new(out_file);
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
    create_tarball(&program_args.mappings, program_args.dest)?;
    Ok(())
}

fn main() {
    let program_args = ProgramArgs::parse();

    if program_args.mappings.is_empty() {
        println!("need at least one mapping");
        exit(-1);
    }

    if program_args.dest.is_empty() {
        println!("invalid destination name");
        exit(-1);
    }

    match try_main(program_args) {
        Ok(()) => {},
        Err(e) => {
            println!("Error: {}", e);
            exit(-1);
        }
    }

    println!("finished");
}
