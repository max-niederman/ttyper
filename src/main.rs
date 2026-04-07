mod calculate;
mod cli;
mod components;
mod config;
mod error;
mod messages;
mod model;
mod resources;

use clap::CommandFactory;
use clap::Parser;
use clap_complete::generate;
use clap_complete::Shell;
use std::io;
use std::time::Duration;
use tuirealm::{
    event::NoUserEvent, terminal::TerminalBridge, Application, EventListenerCfg, PollStrategy,
    Update,
};

use crate::cli::{Command, Opt};
use crate::components::test::{Test, TestComponent};
use crate::components::Screen as Id;
use crate::config::Config;
use crate::error::TtyperError;
use crate::messages::Msg;
use crate::model::Model;

fn list_languages(opt: &Opt) -> eyre::Result<()> {
    opt.languages().map_err(TtyperError::Io)?.for_each(|name| {
        if let Some(s) = name.to_str() {
            println!("{}", s);
        }
    });

    Ok(())
}

fn generate_completions(shell: Shell) -> eyre::Result<()> {
    generate(shell, &mut Opt::command(), "ttyper", &mut io::stdout());
    Ok(())
}

fn make_test(opt: &Opt) -> eyre::Result<Vec<String>> {
    let contents = opt.gen_contents().ok_or_else(|| {
        TtyperError::Content(
            "Couldn't get test contents. Make sure the specified language actually exists.".into(),
        )
    })?;

    if contents.is_empty() {
        return Err(eyre::eyre!("Error: the provided file or language contains no words to type. If you specified a file, make sure it isn't empty."));
    }

    Ok(contents)
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
    let options = Opt::parse();
    let config = options.config();

    if let Some(Command::Completions { shell }) = options.command {
        generate_completions(shell)?;
        return Ok(());
    }

    if options.list_languages {
        list_languages(&options)?;
        return Ok(());
    }

    let contents = make_test(&options)?;

    let (app, terminal_bridge) = setup_ttyper(contents, &options, &config)?;

    let model = Model::new(app, terminal_bridge, config, options);

    event_loop(model)?;

    Ok(())
}

fn setup_ttyper(
    contents: Vec<String>,
    options: &Opt,
    config: &Config,
) -> eyre::Result<(
    Application<Id, Msg, NoUserEvent>,
    TerminalBridge<tuirealm::terminal::CrosstermTerminalAdapter>,
)> {
    let terminal_bridge =
        TerminalBridge::init_crossterm().map_err(|e| TtyperError::Terminal(e.to_string()))?;

    let mut app: Application<Id, Msg, NoUserEvent> = Application::init(
        EventListenerCfg::<NoUserEvent>::default()
            .with_handle(tokio::runtime::Handle::current())
            .async_crossterm_input_listener(Duration::from_millis(0), 1)
            .tick_interval(Duration::from_secs(1))
            .async_tick(true),
    );

    app.mount(
        Id::Test,
        Box::new(TestComponent {
            test: Test::new(
                contents,
                !options.no_backtrack,
                options.sudden_death,
                !options.no_backspace,
            ),
            theme: config.theme.clone(),
        }),
        vec![],
    )
    .map_err(|e| TtyperError::Application(e.to_string()))?;

    // Enable focus
    app.active(&Id::Test)
        .map_err(|e| TtyperError::Application(e.to_string()))?;

    Ok((app, terminal_bridge))
}

fn event_loop(mut model: Model) -> eyre::Result<()> {
    // Terminal initialization (Alternate screen, Raw mode) is already handled by
    let _ = model.terminal.clear_screen();

    while !model.quit {
        // Tick
        match model.app.tick(PollStrategy::BlockCollectUpTo(10)) {
            Err(err) => {
                let _ = model.terminal.restore();
                return Err(TtyperError::Application(err.to_string()).into());
            }
            Ok(messages) => {
                if !messages.is_empty() {
                    model.redraw = true;
                    for msg in messages.into_iter() {
                        let mut msg = Some(msg);
                        while msg.is_some() {
                            msg = model.update(msg);
                        }
                    }
                }
            }
        }

        // Redraw
        if model.redraw {
            model.view()?;
            model.redraw = false;
        }
    }

    // Restore terminal
    let _ = model.terminal.restore();

    Ok(())
}
