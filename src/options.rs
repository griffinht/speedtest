pub fn matches(arguments: Vec<String>) -> std::result::Result<getopts::Matches, i32> {

    let mut options = getopts::Options::new();

    options.optflag("h", "help", "print help");
    options.optflag("v", "version", "print version");
    options.optopt("c", "client", "run as client, connect to address", "<address>");
    options.optopt("s", "server", format!("run as server, bind to address (default {})", crate::default_bind_address!()).as_str(), "<address>");

    if arguments.len() == 0 { eprintln!("missing program name from argv"); return Err(1); }
    let matches = match options.parse(&arguments[1..]) {
        Ok(matches) => matches,
        Err(fail) => { eprintln!("{}", fail); return Err(1); }
    };
    if matches.opt_present("h") {
        eprint!("{}", options.usage_with_format(|opts| {
            format!(
                concat!("Usage: ", env!("CARGO_PKG_NAME"), " [options...]\n{}\n"),
                opts.collect::<Vec<String>>().join("\n")
            )
        }));
        return Err(0)
    }
    if matches.opt_present("v") {
        eprintln!(concat!(env!("CARGO_PKG_NAME"), " version ", env!("CARGO_PKG_VERSION")));
        return Err(0)
    }

    Ok(matches)
}