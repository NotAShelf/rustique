fn main() {
   let args: Vec<String> = std::env::args().collect();
   let bin_name = args.first().map(String::as_str).unwrap_or("");

   // GUI mode when: binary name ends with "gui", --gui flag present, or no subcommand given.
   let gui_mode = args.len() == 1 || bin_name.ends_with("gui") || args.iter().any(|a| a == "--gui");

   if gui_mode {
      std::process::exit(match lithic_gui::run() {
         Ok(()) => 0,
         Err(_) => 1,
      });
   }

   color_eyre::install().expect("Failed to install color_eyre");
   let rt = tokio::runtime::Builder::new_multi_thread()
      .enable_all()
      .build()
      .expect("Failed to create Tokio runtime");
   if let Err(e) = rt.block_on(lithic_cli::run()) {
      eprintln!("{e:#}");
      std::process::exit(1);
   }
}
