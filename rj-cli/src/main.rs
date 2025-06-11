use std::io::{Read, stdin};

fn main() {
    #[rustfmt::skip]
    let cmd = clap::Command::new("rj")
        .arg(clap::Arg::new("json"))
        .arg(clap::Arg::new("pretty")
            .short('p')
            .long("pretty")
            .action(clap::ArgAction::SetTrue),
        );

    let m = cmd.try_get_matches().unwrap_or_else(|e| e.exit());
    let json: String = m
        .get_one("json")
        .map(|x: &String| x.to_string())
        .unwrap_or_else(|| {
            let mut buf = Vec::new();
            let mut handle = stdin().lock();
            let _ = handle.read_to_end(&mut buf);
            String::from_utf8_lossy(&buf).to_string()
        });
    if m.get_flag("pretty") {
        let formatted = rj::format(&json);
        println!("{formatted}");
    } else {
        let parsed = rj::parse(&json);
        println!("{:#?}", parsed);
    }
}
