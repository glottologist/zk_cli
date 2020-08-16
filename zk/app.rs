use std::io::Write;
use clap::App;

pub struct Environment<'a> {
    output: &'a mut dyn Write,
}

impl Environment<'_> {
    fn new<'a, W: Write>(w: &'a mut W) -> Environment<'a> {
        Environment {
            output: w,
        }
    }
}

// TODO: consider error_chain crate?
#[derive(Debug)]
pub enum ZkCommandExecutionErrorKind {
    ArgumentParsing(clap::Error),
    UnknownCommand(String)
}

pub trait ZkCommand {
    fn new() -> Self where Self: Sized;
    fn name(&self) -> &'static str;
    fn run(&self, env: &mut Environment) -> Result<(), ZkCommandExecutionErrorKind>;
    fn build_clap_subcmd(&self) -> App<'static, 'static>;
}

pub struct ZkApp<'a> {
    args: Vec<String>,
    env: Environment<'a>,
    commands: Vec<Box<dyn ZkCommand>>,
    clap_app: App<'a, 'a>,
}

impl ZkApp<'_> {
    pub fn new<'a, W: Write>(args: Vec<String>, w: &'a mut W) -> ZkApp<'a> {
        let app = App::new("zk_cli")
                      .author("Stepan Repin <stnrepin@gmail.com>")
                      .about("A CLI note-taking application for \
                              Zettelkasten methodology.");
        ZkApp {
            args,
            env: Environment::new(w),
            commands: Vec::new(),
            clap_app: app,
        }
    }

    pub fn add_command<T: ZkCommand + 'static>(&mut self) {
        self.commands.push(Box::new(T::new())); 
        let associated_subcmd = self.commands.last().unwrap().as_ref().build_clap_subcmd();
        // Yes, clone() and assignment are terrible but I can't do anything due to clap's design.
        // See: https://github.com/clap-rs/clap/issues/1516
        self.clap_app = self.clap_app.clone().subcommand(associated_subcmd);
    }

    pub fn run(&mut self) -> Result<(), ZkCommandExecutionErrorKind> {
        let matches = self.clap_app
            .get_matches_from_safe_borrow(self.args.clone())
            .map_err(|err| ZkCommandExecutionErrorKind::ArgumentParsing(err))?;

        match matches.subcommand {
            Some(clap_cmd) => {
                match self.commands.iter().find(|x| x.name() == clap_cmd.as_ref().name) {
                    Some(cmd) => cmd.run(&mut self.env),
                    _ => Err(ZkCommandExecutionErrorKind::UnknownCommand("command is unknown".to_string()))
                }
            },
            _ => Err(ZkCommandExecutionErrorKind::UnknownCommand("zk_cli command is not specified".to_string())),
        }
    }
}

#[cfg(test)]
mod test {
    use clap::{App, SubCommand};
    use super::{ZkCommand, ZkCommandExecutionErrorKind, ZkApp, Environment};
    
    //
    // TODO: There are a lot of duplicated code in tests. I tried to get rid of
    // it but failed due to borrow checker errors.
    //
    #[test]
    fn app_new_copy_args() {
        let mut out_buf = Vec::new();
        let args = vec!["1".to_string(), "2".to_string()];
        let app = ZkApp::new(args.clone(), &mut out_buf);
        assert_eq!(app.args, args);
    }

    #[test]
    fn app_new_set_write_reference_to_environment() {
        let mut out_buf: Vec<u8> = Vec::new();
        let args = vec![];
        let app = ZkApp::new(args, &mut out_buf);

        write!(app.env.output, "test-test").unwrap();

        let output = String::from_utf8(out_buf).unwrap();
        assert_eq!(output, "test-test");
    }
    
    #[test]
    fn app_new_creates_empty_command_vector() {
        let mut out_buf: Vec<u8> = Vec::new();
        let args = vec![];
        let app = ZkApp::new(args, &mut out_buf);

        write!(app.env.output, "test-test").unwrap();

        let output = String::from_utf8(out_buf).unwrap();
        assert_eq!(output, "test-test");
    }

    #[test]
    fn app_new_creates_with_about() {
        let mut output = std::io::stdout();
        let args = vec![];
        let mut app = ZkApp::new(args, &mut output);
        let mut out_buf: Vec<u8> = Vec::new();

        app.clap_app.write_long_help(&mut out_buf).unwrap();

        let output = String::from_utf8(out_buf).unwrap();
        assert!(output.contains("zk_cli"));
        assert!(output.contains("Stepan Repin <stnrepin@gmail.com>"));
        assert!(output.contains("Zettelkasten"));
    }

    struct TestCommand;
    
    impl ZkCommand for TestCommand {
        fn new() -> TestCommand {
            TestCommand {}
        }

        fn name(&self) -> &'static str {
            "test"
        }

        fn run(&self, env: &mut Environment) -> Result<(), ZkCommandExecutionErrorKind> {
            write!(env.output, "test").unwrap();
            Ok(())
        }

        fn build_clap_subcmd(&self) -> App<'static, 'static> {
            SubCommand::with_name("test")
        }
    }

    #[test]
    fn app_add_command_adds_command() {
        let mut output = std::io::stdout();
        let args = vec!["zk".to_string()];
        let mut app = ZkApp::new(args, &mut output);

        app.add_command::<TestCommand>();

        assert_eq!(app.commands.len(), 1);
        assert_eq!(app.commands[0].name(), "test");
    }

    #[test]
    fn app_add_command_update_claps_subcommands() {
        let mut output = std::io::stdout();
        let args = vec!["zk".to_string()];
        let mut app = ZkApp::new(args, &mut output);

        app.add_command::<TestCommand>();

        assert!(
            match app.clap_app.get_matches_from_safe(&["zk", "test"]).unwrap().subcommand {
                Some(_) => true,
                _ => false,
            }
        )
    }

    #[test]
    fn app_run_read_args_and_executes_command() {
        let mut out_buf: Vec<u8> = Vec::new();
        let args = vec!["zk".to_string(), "test".to_string()];
        let mut app = ZkApp::new(args, &mut out_buf);
        app.add_command::<TestCommand>();

        app.run().unwrap();

        let output = String::from_utf8(out_buf).unwrap();
        assert_eq!(output, "test");
    }
}
