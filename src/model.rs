use tuirealm::ratatui::layout::{Constraint, Direction, Layout};
use tuirealm::terminal::CrosstermTerminalAdapter;
use tuirealm::{event::NoUserEvent, terminal::TerminalBridge, Application, Update};

use crate::cli::Opt;
use crate::components::result::ResultsComponent;
use crate::components::test::{Test, TestComponent};
use crate::components::Screen as Id;
use crate::config::Config;
use crate::error::Result;
use crate::messages::Msg;

pub struct Model {
    pub app: Application<Id, Msg, NoUserEvent>,
    pub quit: bool,
    pub redraw: bool,
    pub terminal: TerminalBridge<CrosstermTerminalAdapter>,
    pub config: Config,
    pub opt: Opt,
}

impl Model {
    pub fn new(
        app: Application<Id, Msg, NoUserEvent>,
        terminal: TerminalBridge<CrosstermTerminalAdapter>,
        config: Config,
        opt: Opt,
    ) -> Self {
        Self {
            app,
            quit: false,
            redraw: true,
            terminal,
            config,
            opt,
        }
    }

    pub fn view(&mut self) -> Result<()> {
        self.terminal
            .draw(|f| {
                let area = f.area();
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Percentage(100)])
                    .split(area);

                // Render the active view
                // We assume only one view is active at a time (Test or Results)
                if self.app.mounted(&Id::Test) {
                    self.app.view(&Id::Test, f, chunks[0]);
                } else if self.app.mounted(&Id::Results) {
                    self.app.view(&Id::Results, f, chunks[0]);
                }
            })
            .map_err(|e| crate::error::TtyperError::Terminal(e.to_string()))?;
        Ok(())
    }
}

impl Update<Msg> for Model {
    fn update(&mut self, msg: Option<Msg>) -> Option<Msg> {
        if let Some(msg) = msg {
            self.redraw = true;
            match msg {
                Msg::AppClose => {
                    self.quit = true;
                    None
                }
                Msg::ShowResults(results) => {
                    // Mount Results component
                    let _ = self.app.umount(&Id::Test);
                    let _ = self.app.mount(
                        Id::Results,
                        Box::new(ResultsComponent {
                            results,
                            theme: self.config.theme.clone(),
                        }),
                        vec![],
                    );
                    let _ = self.app.active(&Id::Results);
                    None
                }
                Msg::RestartTest => {
                    // Generate new content
                    let contents = self.opt.gen_contents().unwrap_or_default();

                    // Mount Test component
                    let _ = self.app.umount(&Id::Results);
                    let _ = self.app.mount(
                        Id::Test,
                        Box::new(TestComponent {
                            test: Test::new(
                                contents,
                                !self.opt.no_backtrack,
                                self.opt.sudden_death,
                                !self.opt.no_backspace,
                            ),
                            theme: self.config.theme.clone(),
                        }),
                        vec![],
                    );
                    let _ = self.app.active(&Id::Test);
                    None
                }
                Msg::StartTest(words) => {
                    // Mount Test component with specific words
                    let _ = self.app.umount(&Id::Results);
                    let _ = self.app.mount(
                        Id::Test,
                        Box::new(TestComponent {
                            test: Test::new(
                                words,
                                !self.opt.no_backtrack,
                                self.opt.sudden_death,
                                !self.opt.no_backspace,
                            ),
                            theme: self.config.theme.clone(),
                        }),
                        vec![],
                    );
                    let _ = self.app.active(&Id::Test);
                    None
                }
                Msg::None => None,
            }
        } else {
            None
        }
    }
}
