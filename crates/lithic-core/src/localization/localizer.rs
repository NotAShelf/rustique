use clap::Arg;

pub fn handle_about_text(cmd: clap::Command) -> clap::Command {
    cmd.mut_arg("verbose", test_verbose)
}

fn test_verbose(arg: Arg) -> Arg {
    arg.help("Shows info level logging messages. This is very noisy, used for debugging purposes.")
}