#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug)]
pub(crate) enum Mode {
    None,
    List,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Args {
    pub path: String,
    pub mode: Mode,
}

impl Args {
    const fn new() -> Self {
        Self {
            path: String::new(),
            mode: Mode::None,
        }
    }

    pub(crate) fn parse(mut args: impl DoubleEndedIterator<Item = String>) -> Self {
        let mut output = Self::new();

        let last = match args.nth_back(0) {
            Some(a) if !a.contains('-') => a,
            _ => {
                eprintln!("insufficient args");
                std::process::exit(0);
            }
        };

        let mut args = args.peekable();
        while let Some(arg) = args.next() {
            if let Some((name, value)) = match arg.contains('=') {
                // `--path=value`
                true => arg.split_once('=').map(|s| (s.0.into(), Some(s.1.into()))),
                // `--path value`
                false if !args.peek().is_some_and(|x| x.contains('-')) => {
                    Some((arg.clone(), Some(args.next().unwrap_or(last.clone()))))
                }
                // `--flag`
                false => Some((arg.clone(), None)),
            } {
                match name.trim_matches('-') {
                    "p" | "path" => {
                        let value = value.unwrap();
                        let pwd = std::env::current_dir().expect("current directory");
                        output.path.push_str(if value == "." || value == "./" {
                            pwd.to_str().unwrap()
                        } else {
                            &value
                        });
                    }
                    "list" => {
                        output.mode = Mode::List;
                    }
                    _ => {}
                };
            }
        }

        if output.path.is_empty() {
            output.path.push_str(&last);
        }

        output
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn into_args<'a>(input: &'a [&str]) -> impl DoubleEndedIterator<Item = String> + 'a {
        input.iter().map(|&x| x.to_string())
    }

    #[test]
    fn parse_args() {
        let path = String::from("./file");
        let ex1 = Args {
            path: path.clone(),
            mode: Mode::List,
        };
        let ex2 = Args {
            path: path.clone(),
            mode: Mode::None,
        };

        assert_eq!(ex1, Args::parse(into_args(&["--list", "--path", "./file"])));
        assert_eq!(ex1, Args::parse(into_args(&["--list", "./file"])));
        assert_eq!(ex2, Args::parse(into_args(&["./file"])));
    }
}
