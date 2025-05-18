use std::env::args;
use crate::commands::arg_structs::download_args::DownloadArgs;
use crate::rustique_errors::RustiqueError;

pub async fn download(args: &DownloadArgs) -> Result<(), RustiqueError> {
    args.validate()?;

    println!("Downloading vintage story executable...");
    println!("{:?}", args);
    
    
    Ok(())
}