use color_eyre::Result;

fn main() -> Result<()> {
   color_eyre::install()?;
   let rt = tokio::runtime::Builder::new_multi_thread()
      .enable_all()
      .build()
      .expect("Failed to create Tokio runtime");
   rt.block_on(lithic_cli::run())
}
