use anyhow::Result;
use chisel_docs::{Doc, DocsService, ListOptions};
use chisel_render::colors::*;
use chisel_specs::{Spec, SpecStatus, SpecsService};
use chisel_store::SpecRow;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap},
    Frame, Terminal,
};
use std::io;
use std::path::PathBuf;

// --- Shared TUI Components & Helpers ---

#[derive(PartialEq, Debug, Clone)]
pub enum TuiAction {
    None,
    Quit,
}

#[derive(Clone)]
enum AppPrompt {
    None,
    Input {
        label: String,
        buffer: String,
        kind: PromptKind,
    },
    Select {
        label: String,
        options: Vec<String>,
        selected: usize,
        kind: PromptKind,
    },
}

#[derive(Clone, Copy)]
enum PromptKind {
    NewDocTitle,
    MoveDocCategory,
    ReorderDocOrder,
    ReorderCategoryOrder,
    NewSpecTitle,
    ChangeSpecStatus,
    ConfirmDeleteSpec,
}

fn preview_content_to_lines(content: &str) -> Vec<Line<'_>> {
    content
        .lines()
        .map(|l| {
            if l.starts_with('#') {
                Line::from(Span::styled(
                    l,
                    Style::default()
                        .fg(ACCENT_YELLOW)
                        .add_modifier(Modifier::BOLD),
                ))
            } else {
                Line::from(l)
            }
        })
        .collect()
}

fn render_footer(f: &mut Frame, area: Rect, spans: Vec<Span>) {
    f.render_widget(Clear, area);
    let footer = Paragraph::new(Line::from(spans)).style(Style::default().bg(PANEL_DARK));
    f.render_widget(footer, area);
}

fn next_list_item(state: &mut ListState, count: usize) {
    if count == 0 {
        return;
    }
    let i = match state.selected() {
        Some(i) => {
            if i >= count - 1 {
                0
            } else {
                i + 1
            }
        }
        None => 0,
    };
    state.select(Some(i));
}

fn prev_list_item(state: &mut ListState, count: usize) {
    if count == 0 {
        return;
    }
    let i = match state.selected() {
        Some(i) => {
            if i == 0 {
                count - 1
            } else {
                i - 1
            }
        }
        None => 0,
    };
    state.select(Some(i));
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

#[derive(Clone, Default)]
struct SearchState {
    buffer: String,
    active: bool,
}

impl SearchState {
    fn handle_key(&mut self, key: event::KeyEvent) -> bool {
        match key.code {
            KeyCode::Char('/') if !self.active => {
                self.active = true;
                self.buffer.clear();
                true
            }
            KeyCode::Char(c) if self.active => {
                self.buffer.push(c);
                true
            }
            KeyCode::Backspace if self.active => {
                self.buffer.pop();
                true
            }
            KeyCode::Enter | KeyCode::Esc if self.active => {
                if key.code == KeyCode::Esc {
                    self.buffer.clear();
                }
                self.active = false;
                true
            }
            _ => false,
        }
    }

    fn render_footer(&self, f: &mut Frame, area: Rect) {
        render_footer(
            f,
            area,
            vec![
                Span::styled(
                    " /",
                    Style::default()
                        .fg(ACCENT_CYAN)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(&self.buffer),
                Span::styled(
                    "█",
                    Style::default()
                        .fg(ACCENT_CYAN)
                        .add_modifier(Modifier::SLOW_BLINK),
                ),
            ],
        );
    }
}

// --- Docs Explorer App ---

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum DocsPane {
    Sidebar,
    Main,
    Preview,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct DisplayCategory {
    order: i32,
    label: String,
    id: String, // Original folder name or GENERAL/[ALL]
}

pub struct DocsApp {
    service: DocsService,
    all_docs: Vec<Doc>,
    categories: Vec<DisplayCategory>,
    sidebar_state: ListState,
    filtered_docs: Vec<Doc>,
    main_list_state: ListState,
    selected_doc: Option<Doc>,
    active_pane: DocsPane,
    preview_scroll: u16,
    prompt: AppPrompt,
    search: SearchState,
    exit_action: TuiAction,
}

impl DocsApp {
    pub async fn new(service: DocsService, initial_path: Option<PathBuf>) -> Result<Self> {
        let mut app = Self {
            service,
            all_docs: Vec::new(),
            categories: Vec::new(),
            sidebar_state: ListState::default(),
            filtered_docs: Vec::new(),
            main_list_state: ListState::default(),
            selected_doc: None,
            active_pane: DocsPane::Sidebar,
            preview_scroll: 0,
            prompt: AppPrompt::None,
            search: SearchState::default(),
            exit_action: TuiAction::None,
        };
        app.refresh_data().await?;
        app.sidebar_state.select(Some(0));

        if let Some(path) = initial_path {
            let resolved = app
                .service
                .source
                .resolve_path(None, path.to_string_lossy().to_string()); // Simple resolve
            app.select_doc_by_path(resolved).await;
        } else {
            app.update_filtered_docs().await;
        }

        Ok(app)
    }

    async fn update_filtered_docs(&mut self) {
        let selected_cat = self
            .sidebar_state
            .selected()
            .and_then(|i| self.categories.get(i))
            .cloned()
            .unwrap_or_else(|| DisplayCategory {
                order: -1,
                label: "[ALL]".to_string(),
                id: "[ALL]".to_string(),
            });

        let mut docs = if selected_cat.id == "[ALL]" {
            self.all_docs.clone()
        } else if selected_cat.id == "GENERAL" {
            self.all_docs
                .iter()
                .filter(|d| d.category.is_none())
                .cloned()
                .collect()
        } else {
            self.all_docs
                .iter()
                .filter(|d| d.category.as_deref() == Some(&selected_cat.id))
                .cloned()
                .collect()
        };

        if !self.search.buffer.is_empty() {
            let query = self.search.buffer.to_lowercase();
            docs.retain(|d| {
                d.name.to_lowercase().contains(&query)
                    || d.frontmatter
                        .as_ref()
                        .map(|f| f.title.to_lowercase().contains(&query))
                        .unwrap_or(false)
            });
        }

        chisel_docs::DocList::sort_docs(&mut docs);

        self.filtered_docs = docs;

        if self.filtered_docs.is_empty() {
            self.main_list_state.select(None);
        } else {
            let current = self.main_list_state.selected().unwrap_or(0);
            if current >= self.filtered_docs.len() {
                self.main_list_state.select(Some(0));
            } else {
                self.main_list_state.select(Some(current));
            }
        }
        self.update_preview().await;
    }

    async fn select_doc_by_path(&mut self, path: PathBuf) {
        if let Some(doc) = self.all_docs.iter().find(|d| d.path == path) {
            let cat_id = doc.category.as_deref().unwrap_or("GENERAL").to_string();
            if let Some(cat_idx) = self.categories.iter().position(|c| c.id == cat_id) {
                self.sidebar_state.select(Some(cat_idx));
                self.update_filtered_docs().await;

                if let Some(doc_idx) = self.filtered_docs.iter().position(|d| d.path == path) {
                    self.main_list_state.select(Some(doc_idx));
                    self.active_pane = DocsPane::Main;
                    self.update_preview().await;
                }
            }
        }
    }

    async fn update_preview(&mut self) {
        if let Some(selected) = self.main_list_state.selected() {
            if let Some(doc_ref) = self.filtered_docs.get(selected) {
                self.selected_doc = self.service.show(doc_ref.path.clone()).await.ok();
                if let Some(doc) = &mut self.selected_doc {
                    doc.category = doc_ref.category.clone();
                }
            }
        } else {
            self.selected_doc = None;
        }
    }

    pub async fn run(&mut self) -> Result<TuiAction> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        let res = self.main_loop(&mut terminal).await;

        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        terminal.show_cursor()?;

        res.map(|_| std::mem::replace(&mut self.exit_action, TuiAction::None))
    }

    async fn main_loop<B: ratatui::backend::Backend + io::Write>(
        &mut self,
        terminal: &mut Terminal<B>,
    ) -> Result<()> {
        loop {
            terminal.draw(|f| self.render(f))?;

            if event::poll(std::time::Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    if self.search.handle_key(key) {
                        self.update_filtered_docs().await;
                    } else if let AppPrompt::None = self.prompt {
                        match key.code {
                            KeyCode::Char('q') | KeyCode::Esc => {
                                self.exit_action = TuiAction::Quit;
                                return Ok(());
                            }
                            KeyCode::Char('[') | KeyCode::Char(']') => match self.active_pane {
                                DocsPane::Sidebar => {
                                    if let Some(selected) = self.sidebar_state.selected() {
                                        if let Some(cat) = self.categories.get(selected) {
                                            if cat.id != "[ALL]" {
                                                self.prompt = AppPrompt::Input {
                                                    label: format!("Order for {}: ", cat.label),
                                                    buffer: cat.order.to_string(),
                                                    kind: PromptKind::ReorderCategoryOrder,
                                                };
                                            }
                                        }
                                    }
                                }
                                DocsPane::Main => {
                                    if let Some(selected) = self.main_list_state.selected() {
                                        if let Some(doc) = self.filtered_docs.get(selected) {
                                            self.prompt = AppPrompt::Input {
                                                label: format!("Order for {}: ", doc.name),
                                                buffer: doc
                                                    .frontmatter
                                                    .as_ref()
                                                    .and_then(|f| f.order)
                                                    .unwrap_or(0)
                                                    .to_string(),
                                                kind: PromptKind::ReorderDocOrder,
                                            };
                                        }
                                    }
                                }
                                _ => {}
                            },
                            KeyCode::Tab => {
                                self.active_pane = match self.active_pane {
                                    DocsPane::Sidebar => DocsPane::Main,
                                    DocsPane::Main => DocsPane::Preview,
                                    DocsPane::Preview => DocsPane::Sidebar,
                                };
                            }
                            KeyCode::BackTab => {
                                self.active_pane = match self.active_pane {
                                    DocsPane::Sidebar => DocsPane::Preview,
                                    DocsPane::Main => DocsPane::Sidebar,
                                    DocsPane::Preview => DocsPane::Main,
                                };
                            }
                            KeyCode::Char('h') | KeyCode::Left => {
                                self.active_pane = match self.active_pane {
                                    DocsPane::Main => DocsPane::Sidebar,
                                    DocsPane::Preview => DocsPane::Main,
                                    DocsPane::Sidebar => DocsPane::Sidebar,
                                };
                            }
                            KeyCode::Char('l') | KeyCode::Right => {
                                self.active_pane = match self.active_pane {
                                    DocsPane::Sidebar => DocsPane::Main,
                                    DocsPane::Main => DocsPane::Preview,
                                    DocsPane::Preview => DocsPane::Preview,
                                };
                            }
                            KeyCode::Char('j') | KeyCode::Down => match self.active_pane {
                                DocsPane::Sidebar => {
                                    next_list_item(&mut self.sidebar_state, self.categories.len());
                                    self.update_filtered_docs().await;
                                }
                                DocsPane::Main => {
                                    next_list_item(
                                        &mut self.main_list_state,
                                        self.filtered_docs.len(),
                                    );
                                    self.update_preview().await;
                                }
                                DocsPane::Preview => {
                                    self.preview_scroll = self.preview_scroll.saturating_add(1);
                                }
                            },
                            KeyCode::Char('k') | KeyCode::Up => match self.active_pane {
                                DocsPane::Sidebar => {
                                    prev_list_item(&mut self.sidebar_state, self.categories.len());
                                    self.update_filtered_docs().await;
                                }
                                DocsPane::Main => {
                                    prev_list_item(
                                        &mut self.main_list_state,
                                        self.filtered_docs.len(),
                                    );
                                    self.update_preview().await;
                                }
                                DocsPane::Preview => {
                                    self.preview_scroll = self.preview_scroll.saturating_sub(1);
                                }
                            },
                            KeyCode::Enter => {
                                if self.active_pane == DocsPane::Sidebar {
                                    self.active_pane = DocsPane::Main;
                                } else if let Some(selected) = self.main_list_state.selected() {
                                    if let Some(doc) = self.filtered_docs.get(selected) {
                                        disable_raw_mode()?;
                                        execute!(
                                            terminal.backend_mut(),
                                            LeaveAlternateScreen,
                                            DisableMouseCapture
                                        )?;
                                        let _ = self.service.edit(doc.path.clone()).await;
                                        enable_raw_mode()?;
                                        execute!(
                                            terminal.backend_mut(),
                                            EnterAlternateScreen,
                                            EnableMouseCapture
                                        )?;
                                        terminal.clear()?;
                                        self.refresh_data().await?;
                                    }
                                }
                            }
                            KeyCode::Char('e') => {
                                if let Some(selected) = self.main_list_state.selected() {
                                    if let Some(doc) = self.filtered_docs.get(selected) {
                                        disable_raw_mode()?;
                                        execute!(
                                            terminal.backend_mut(),
                                            LeaveAlternateScreen,
                                            DisableMouseCapture
                                        )?;
                                        let _ = self.service.edit(doc.path.clone()).await;
                                        enable_raw_mode()?;
                                        execute!(
                                            terminal.backend_mut(),
                                            EnterAlternateScreen,
                                            EnableMouseCapture
                                        )?;
                                        terminal.clear()?;
                                        self.refresh_data().await?;
                                    }
                                }
                            }
                            KeyCode::Char('n') => {
                                self.prompt = AppPrompt::Input {
                                    label: "Document Title: ".to_string(),
                                    buffer: String::new(),
                                    kind: PromptKind::NewDocTitle,
                                };
                            }
                            KeyCode::Char('m') => {
                                if let Some(selected) = self.main_list_state.selected() {
                                    if let Some(doc) = self.filtered_docs.get(selected) {
                                        self.prompt = AppPrompt::Input {
                                            label: "Category: ".to_string(),
                                            buffer: doc.category.clone().unwrap_or_default(),
                                            kind: PromptKind::MoveDocCategory,
                                        };
                                    }
                                }
                            }
                            _ => {}
                        }
                    } else {
                        // Handle Prompt Input
                        match &mut self.prompt {
                            AppPrompt::Input { buffer, kind, .. } => match key.code {
                                KeyCode::Char(c) => buffer.push(c),
                                KeyCode::Backspace => {
                                    buffer.pop();
                                }
                                KeyCode::Esc => {
                                    self.prompt = AppPrompt::None;
                                }
                                KeyCode::Enter => {
                                    let input = buffer.clone();
                                    let kind = *kind;
                                    self.prompt = AppPrompt::None;
                                    self.handle_prompt_confirm(input, kind).await?;
                                }
                                _ => {}
                            },
                            AppPrompt::Select {
                                selected,
                                options,
                                kind,
                                ..
                            } => match key.code {
                                KeyCode::Char('j') | KeyCode::Down => {
                                    if *selected < options.len() - 1 {
                                        *selected += 1;
                                    }
                                }
                                KeyCode::Char('k') | KeyCode::Up => {
                                    if *selected > 0 {
                                        *selected -= 1;
                                    }
                                }
                                KeyCode::Esc => {
                                    self.prompt = AppPrompt::None;
                                }
                                KeyCode::Enter => {
                                    let idx = *selected;
                                    let kind = *kind;
                                    self.prompt = AppPrompt::None;
                                    self.handle_select_confirm(idx, kind).await?;
                                }
                                _ => {}
                            },
                            _ => {}
                        }
                    }
                }
            }
        }
    }

    async fn handle_prompt_confirm(&mut self, input: String, kind: PromptKind) -> Result<()> {
        match kind {
            PromptKind::NewDocTitle => {
                if !input.is_empty() {
                    let _ = self.service.create(&input, None).await?;
                    self.refresh_data().await?;
                }
            }
            PromptKind::MoveDocCategory => {
                if let Some(selected) = self.main_list_state.selected() {
                    if let Some(doc) = self.filtered_docs.get(selected) {
                        let cat_opt = if input.is_empty() { None } else { Some(input) };
                        let _ = self.service.move_doc(doc.path.clone(), cat_opt).await?;
                        self.refresh_data().await?;
                    }
                }
            }
            PromptKind::ReorderDocOrder => {
                if let Ok(order) = input.parse::<i32>() {
                    if let Some(selected) = self.main_list_state.selected() {
                        if let Some(doc) = self.filtered_docs.get(selected) {
                            self.service
                                .update_doc_order(doc.path.clone(), order)
                                .await?;
                            self.refresh_data().await?;
                        }
                    }
                }
            }
            PromptKind::ReorderCategoryOrder => {
                if let Ok(order) = input.parse::<i32>() {
                    if let Some(selected) = self.sidebar_state.selected() {
                        if let Some(cat) = self.categories.get(selected) {
                            self.service.update_category_order(&cat.id, order)?;
                            self.refresh_data().await?;
                        }
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }

    async fn handle_select_confirm(&mut self, _idx: usize, _kind: PromptKind) -> Result<()> {
        Ok(())
    }

    async fn refresh_data(&mut self) -> Result<()> {
        let docs = self
            .service
            .list(ListOptions {
                root: self.service.workspace_root.clone(),
                use_gitignore: true,
                include_hidden: false,
            })
            .await?;
        self.all_docs = docs.0;

        let mut categories_map = std::collections::BTreeMap::new();

        // Always include [ALL]
        categories_map.insert(
            "[ALL]".to_string(),
            DisplayCategory {
                order: -1,
                label: "[ALL]".to_string(),
                id: "[ALL]".to_string(),
            },
        );

        for doc in &self.all_docs {
            let cat_id = doc.category.as_deref().unwrap_or("GENERAL").to_string();
            if !categories_map.contains_key(&cat_id) {
                // Category metadata is still direct FS for now
                // Ideally we'd move this into DataSource
                let meta = chisel_docs::get_category_metadata(
                    &self.service.workspace_root.join(".chisel").join("docs"),
                    &cat_id,
                );
                categories_map.insert(
                    cat_id.clone(),
                    DisplayCategory {
                        order: meta.order.unwrap_or(i32::MAX),
                        label: meta.label.unwrap_or_else(|| cat_id.clone()),
                        id: cat_id,
                    },
                );
            }
        }

        let mut categories: Vec<DisplayCategory> = categories_map.into_values().collect();
        categories.sort();
        self.categories = categories;

        self.update_filtered_docs().await;
        Ok(())
    }

    fn render(&mut self, f: &mut Frame) {
        f.render_widget(Clear, f.area());

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0), Constraint::Length(1)])
            .split(f.area());
        let main_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(20),
                Constraint::Percentage(30),
                Constraint::Percentage(50),
            ])
            .split(chunks[0]);

        // Sidebar
        let sidebar_block = Block::default()
            .borders(Borders::ALL)
            .title(" CATEGORIES ")
            .border_style(if self.active_pane == DocsPane::Sidebar {
                Style::default().fg(ACCENT_BLUE)
            } else {
                Style::default().fg(Color::DarkGray)
            });
        let sidebar_items: Vec<ListItem> = self
            .categories
            .iter()
            .map(|cat| ListItem::new(Span::raw(&cat.label)))
            .collect();
        f.render_stateful_widget(
            List::new(sidebar_items)
                .block(sidebar_block)
                .highlight_style(Style::default().bg(PANEL_DARK).add_modifier(Modifier::BOLD))
                .highlight_symbol("▸ "),
            main_chunks[0],
            &mut self.sidebar_state,
        );

        // Main List
        let main_block = Block::default()
            .borders(Borders::ALL)
            .title(" DOCUMENTS ")
            .border_style(if self.active_pane == DocsPane::Main {
                Style::default().fg(ACCENT_BLUE)
            } else {
                Style::default().fg(Color::DarkGray)
            });
        let main_items: Vec<ListItem> = self
            .filtered_docs
            .iter()
            .map(|doc| ListItem::new(Span::raw(&doc.name)))
            .collect();
        f.render_stateful_widget(
            List::new(main_items)
                .block(main_block)
                .highlight_style(Style::default().bg(PANEL_DARK).add_modifier(Modifier::BOLD))
                .highlight_symbol("▸ "),
            main_chunks[1],
            &mut self.main_list_state,
        );

        // Preview
        let preview_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0), Constraint::Length(8)])
            .split(main_chunks[2]);
        let preview_border_style = if self.active_pane == DocsPane::Preview {
            Style::default().fg(ACCENT_BLUE)
        } else {
            Style::default().fg(Color::DarkGray)
        };
        let content_text = self
            .selected_doc
            .as_ref()
            .and_then(|d| d.content.as_deref())
            .unwrap_or("Select a document to preview");
        f.render_widget(
            Paragraph::new(preview_content_to_lines(content_text))
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(" CONTENT ")
                        .border_style(preview_border_style),
                )
                .wrap(Wrap { trim: false })
                .scroll((self.preview_scroll, 0)),
            preview_chunks[0],
        );

        let metadata_block = Block::default()
            .borders(Borders::ALL)
            .title(" METADATA ")
            .border_style(preview_border_style);
        if let Some(doc) = &self.selected_doc {
            let title = doc
                .frontmatter
                .as_ref()
                .map(|f| f.title.as_str())
                .unwrap_or(&doc.name);
            let tags = doc
                .frontmatter
                .as_ref()
                .map(|f| f.tags.join(", "))
                .unwrap_or_else(|| "none".to_string());
            let updated = doc.updated_at.format("%Y-%m-%d %H:%M").to_string();
            let metadata_text = Text::from(vec![
                Line::from(vec![
                    Span::styled(" TITLE: ", Style::default().fg(TEXT_DIM)),
                    Span::styled(
                        title,
                        Style::default()
                            .fg(ACCENT_BLUE)
                            .add_modifier(Modifier::BOLD),
                    ),
                ]),
                Line::from(vec![
                    Span::styled(" TAGS:  ", Style::default().fg(TEXT_DIM)),
                    Span::styled(tags, Style::default().fg(ACCENT_MAGENTA)),
                ]),
                Line::from(vec![
                    Span::styled(" SYNC:  ", Style::default().fg(TEXT_DIM)),
                    Span::styled(updated, Style::default().fg(ACCENT_GREEN)),
                ]),
            ]);
            f.render_widget(
                Paragraph::new(metadata_text).block(metadata_block),
                preview_chunks[1],
            );
        } else {
            f.render_widget(
                Paragraph::new("No metadata available").block(metadata_block),
                preview_chunks[1],
            );
        }

        match &self.prompt {
            AppPrompt::None => {
                if self.search.active {
                    self.search.render_footer(f, chunks[1]);
                } else {
                    render_footer(
                        f,
                        chunks[1],
                        vec![
                            Span::styled(
                                " q",
                                Style::default().fg(ACCENT_RED).add_modifier(Modifier::BOLD),
                            ),
                            Span::raw(":quit "),
                            Span::styled(
                                " /",
                                Style::default()
                                    .fg(ACCENT_CYAN)
                                    .add_modifier(Modifier::BOLD),
                            ),
                            Span::raw(":find "),
                            Span::styled(
                                " [ ]",
                                Style::default()
                                    .fg(ACCENT_YELLOW)
                                    .add_modifier(Modifier::BOLD),
                            ),
                            Span::raw(":reorder "),
                            Span::styled(
                                " n",
                                Style::default()
                                    .fg(ACCENT_BLUE)
                                    .add_modifier(Modifier::BOLD),
                            ),
                            Span::raw(":new "),
                            Span::styled(
                                " e",
                                Style::default()
                                    .fg(ACCENT_GREEN)
                                    .add_modifier(Modifier::BOLD),
                            ),
                            Span::raw(":edit "),
                            Span::styled(
                                " m",
                                Style::default()
                                    .fg(ACCENT_MAGENTA)
                                    .add_modifier(Modifier::BOLD),
                            ),
                            Span::raw(":move "),
                            Span::styled(
                                " Tab",
                                Style::default()
                                    .fg(ACCENT_CYAN)
                                    .add_modifier(Modifier::BOLD),
                            ),
                            Span::raw(":switch "),
                        ],
                    );
                }
            }
            AppPrompt::Input { label, buffer, .. } => {
                render_footer(
                    f,
                    chunks[1],
                    vec![
                        Span::styled(
                            label,
                            Style::default()
                                .fg(ACCENT_CYAN)
                                .add_modifier(Modifier::BOLD),
                        ),
                        Span::raw(buffer),
                        Span::styled(
                            "█",
                            Style::default()
                                .fg(ACCENT_CYAN)
                                .add_modifier(Modifier::SLOW_BLINK),
                        ),
                    ],
                );
            }
            AppPrompt::Select {
                label,
                options,
                selected,
                ..
            } => {
                render_footer(
                    f,
                    chunks[1],
                    vec![
                        Span::styled(
                            " ESC",
                            Style::default().fg(ACCENT_RED).add_modifier(Modifier::BOLD),
                        ),
                        Span::raw(":cancel "),
                    ],
                );

                let area = centered_rect(40, 40, f.area());
                f.render_widget(Clear, area);
                let items: Vec<ListItem> = options
                    .iter()
                    .enumerate()
                    .map(|(i, opt)| {
                        if i == *selected {
                            ListItem::new(Span::styled(
                                format!("▸ {}", opt),
                                Style::default()
                                    .fg(ACCENT_BLUE)
                                    .add_modifier(Modifier::BOLD),
                            ))
                        } else {
                            ListItem::new(Span::raw(format!("  {}", opt)))
                        }
                    })
                    .collect();
                f.render_widget(
                    List::new(items).block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title(label.as_str())
                            .border_style(Style::default().fg(ACCENT_MAGENTA)),
                    ),
                    area,
                );
            }
        }
    }
}

// --- Specs Explorer App ---

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum SpecsPane {
    List,
    Preview,
}


pub struct SpecsApp {
    service: SpecsService,
    all_specs: Vec<SpecRow>,
    filtered_specs: Vec<SpecRow>,
    list_state: ListState,
    selected_spec: Option<Spec>,
    active_pane: SpecsPane,
    preview_scroll: u16,
    prompt: AppPrompt,
    search: SearchState,
    exit_action: TuiAction,
    status_filter: Option<SpecStatus>,
}

impl SpecsApp {
    pub async fn new(
        service: SpecsService,
        status_filter: Option<SpecStatus>,
    ) -> Result<Self> {
        let mut app = Self {
            service,
            all_specs: Vec::new(),
            filtered_specs: Vec::new(),
            list_state: ListState::default(),
            selected_spec: None,
            active_pane: SpecsPane::List,
            preview_scroll: 0,
            prompt: AppPrompt::None,
            search: SearchState::default(),
            exit_action: TuiAction::None,
            status_filter,
        };
        app.refresh_data().await?;
        if !app.filtered_specs.is_empty() {
            app.list_state.select(Some(0));
        }
        app.update_preview().await;
        Ok(app)
    }

    async fn refresh_data(&mut self) -> Result<()> {
        let list = self.service.list(self.status_filter.clone()).await?;
        self.all_specs = list.0;
        self.update_filtered_specs();
        Ok(())
    }

    fn update_filtered_specs(&mut self) {
        let mut filtered = self.all_specs.clone();

        if !self.search.buffer.is_empty() {
            let query = self.search.buffer.to_lowercase();
            filtered.retain(|s| {
                s.title.to_lowercase().contains(&query)
                    || s.slug.to_lowercase().contains(&query)
                    || s.area
                        .as_ref()
                        .map(|a| a.to_lowercase().contains(&query))
                        .unwrap_or(false)
                    || s.excerpt.to_lowercase().contains(&query)
            });
        }

        self.filtered_specs = filtered;
    }

    async fn update_preview(&mut self) {
        let slug = self
            .list_state
            .selected()
            .and_then(|i| self.filtered_specs.get(i))
            .map(|r| r.slug.clone());

        if let Some(slug) = slug {
            self.selected_spec = self.service.show(&slug).await.ok();
            self.preview_scroll = 0;
        } else {
            self.selected_spec = None;
        }
    }

    pub async fn run(&mut self) -> Result<TuiAction> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        let res = self.main_loop(&mut terminal).await;

        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        terminal.show_cursor()?;

        res.map(|_| std::mem::replace(&mut self.exit_action, TuiAction::None))
    }

    async fn main_loop<B: ratatui::backend::Backend + io::Write>(
        &mut self,
        terminal: &mut Terminal<B>,
    ) -> Result<()> {
        loop {
            terminal.draw(|f| self.render(f))?;

            if event::poll(std::time::Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    if self.search.handle_key(key) {
                        self.update_filtered_specs();
                        self.update_preview().await;
                        continue;
                    }

                    match &self.prompt {
                        AppPrompt::None => match key.code {
                            KeyCode::Char('q') | KeyCode::Esc => {
                                self.exit_action = TuiAction::Quit;
                                return Ok(());
                            }
                            KeyCode::Tab => {
                                self.active_pane = match self.active_pane {
                                    SpecsPane::List => SpecsPane::Preview,
                                    SpecsPane::Preview => SpecsPane::List,
                                };
                            }
                            KeyCode::Char('j') | KeyCode::Down => {
                                if self.active_pane == SpecsPane::List {
                                    next_list_item(
                                        &mut self.list_state,
                                        self.filtered_specs.len(),
                                    );
                                    self.update_preview().await;
                                } else {
                                    self.preview_scroll =
                                        self.preview_scroll.saturating_add(3);
                                }
                            }
                            KeyCode::Char('k') | KeyCode::Up => {
                                if self.active_pane == SpecsPane::List {
                                    prev_list_item(
                                        &mut self.list_state,
                                        self.filtered_specs.len(),
                                    );
                                    self.update_preview().await;
                                } else {
                                    self.preview_scroll =
                                        self.preview_scroll.saturating_sub(3);
                                }
                            }
                            KeyCode::Char('n') => {
                                self.prompt = AppPrompt::Input {
                                    label: "New Spec Title".to_string(),
                                    buffer: String::new(),
                                    kind: PromptKind::NewSpecTitle,
                                };
                            }
                            KeyCode::Char('s') => {
                                if self.selected_spec.is_some() {
                                    self.prompt = AppPrompt::Select {
                                        label: "Change Status".to_string(),
                                        options: vec![
                                            "draft".to_string(),
                                            "ready".to_string(),
                                            "in-progress".to_string(),
                                            "shipped".to_string(),
                                            "archived".to_string(),
                                        ],
                                        selected: 0,
                                        kind: PromptKind::ChangeSpecStatus,
                                    };
                                }
                            }
                            KeyCode::Char('e') => {
                                if let Some(spec) = &self.selected_spec {
                                    let slug = spec.slug.clone();
                                    // Exit TUI, edit, re-enter
                                    disable_raw_mode()?;
                                    execute!(
                                        terminal.backend_mut(),
                                        LeaveAlternateScreen,
                                        DisableMouseCapture
                                    )?;
                                    terminal.show_cursor()?;

                                    let _ = self.service.edit(&slug).await;

                                    enable_raw_mode()?;
                                    execute!(
                                        io::stdout(),
                                        EnterAlternateScreen,
                                        EnableMouseCapture
                                    )?;
                                    self.refresh_data().await?;
                                    self.update_preview().await;
                                }
                            }
                            KeyCode::Char('x') if self.selected_spec.is_some() => {
                                self.prompt = AppPrompt::Select {
                                    label: "Delete this spec?".to_string(),
                                    options: vec![
                                        "No".to_string(),
                                        "Yes, delete".to_string(),
                                    ],
                                    selected: 0,
                                    kind: PromptKind::ConfirmDeleteSpec,
                                };
                            }
                            _ => {}
                        },
                        AppPrompt::Input { kind, .. } => {
                            let kind = *kind;
                            match key.code {
                                KeyCode::Char(c) => {
                                    if let AppPrompt::Input { buffer, .. } = &mut self.prompt {
                                        buffer.push(c);
                                    }
                                }
                                KeyCode::Backspace => {
                                    if let AppPrompt::Input { buffer, .. } = &mut self.prompt {
                                        buffer.pop();
                                    }
                                }
                                KeyCode::Enter => {
                                    let buffer = if let AppPrompt::Input { buffer, .. } =
                                        &self.prompt
                                    {
                                        buffer.clone()
                                    } else {
                                        String::new()
                                    };
                                    self.prompt = AppPrompt::None;

                                    if !buffer.is_empty() {
                                        if let PromptKind::NewSpecTitle = kind {
                                            let _ = self
                                                .service
                                                .create(
                                                    &buffer,
                                                    None,
                                                    Some("feature"),
                                                    None,
                                                )
                                                .await;
                                            self.refresh_data().await?;
                                            self.update_preview().await;
                                        }
                                    }
                                }
                                KeyCode::Esc => {
                                    self.prompt = AppPrompt::None;
                                }
                                _ => {}
                            }
                        }
                        AppPrompt::Select { options, kind, .. } => {
                            let kind = *kind;
                            let count = options.len();
                            match key.code {
                                KeyCode::Char('j') | KeyCode::Down => {
                                    if let AppPrompt::Select { selected, .. } = &mut self.prompt
                                    {
                                        *selected = (*selected + 1) % count;
                                    }
                                }
                                KeyCode::Char('k') | KeyCode::Up => {
                                    if let AppPrompt::Select { selected, .. } = &mut self.prompt
                                    {
                                        *selected = if *selected == 0 {
                                            count - 1
                                        } else {
                                            *selected - 1
                                        };
                                    }
                                }
                                KeyCode::Enter => {
                                    let selected = if let AppPrompt::Select {
                                        selected,
                                        options,
                                        ..
                                    } = &self.prompt
                                    {
                                        Some((*selected, options.clone()))
                                    } else {
                                        None
                                    };
                                    self.prompt = AppPrompt::None;

                                    if let Some((idx, opts)) = selected {
                                        match kind {
                                            PromptKind::ChangeSpecStatus => {
                                                if let Some(spec) = &self.selected_spec {
                                                    let slug = spec.slug.clone();
                                                    if let Ok(status) =
                                                        std::str::FromStr::from_str(&opts[idx])
                                                    {
                                                        let _ = self
                                                            .service
                                                            .update_status(&slug, status)
                                                            .await;
                                                        self.refresh_data().await?;
                                                        self.update_preview().await;
                                                    }
                                                }
                                            }
                                            PromptKind::ConfirmDeleteSpec if idx == 1 => {
                                                if let Some(spec) = &self.selected_spec {
                                                    let slug = spec.slug.clone();
                                                    let _ =
                                                        self.service.delete(&slug).await;
                                                    self.refresh_data().await?;
                                                    // Fix selection after delete
                                                    let len =
                                                        self.filtered_specs.len();
                                                    if len == 0 {
                                                        self.list_state
                                                            .select(None);
                                                    } else if let Some(sel) =
                                                        self.list_state.selected()
                                                    {
                                                        if sel >= len {
                                                            self.list_state
                                                                .select(Some(len - 1));
                                                        }
                                                    }
                                                    self.update_preview().await;
                                                }
                                            }
                                            _ => {}
                                        }
                                    }
                                }
                                KeyCode::Esc => {
                                    self.prompt = AppPrompt::None;
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
        }
    }

    fn render(&self, f: &mut Frame) {
        let size = f.area();

        // Main layout: list (40%) | preview (60%)
        let outer = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(3), Constraint::Length(1)])
            .split(size);

        let main_area = outer[0];
        let footer_area = outer[1];

        let columns = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
            .split(main_area);

        // --- Left: Spec List ---
        let list_active = self.active_pane == SpecsPane::List;
        let list_border_color = if list_active { ACCENT_BLUE } else { Color::DarkGray };

        let title = match &self.status_filter {
            Some(s) => format!(" Specs [{}] ", s),
            None => " Specs ".to_string(),
        };

        let items: Vec<ListItem> = self
            .filtered_specs
            .iter()
            .map(|s| {
                let status_color = match s.status.as_str() {
                    "draft" => TEXT_DIM,
                    "ready" => ACCENT_BLUE,
                    "in-progress" => ACCENT_YELLOW,
                    "shipped" => ACCENT_GREEN,
                    "archived" => TEXT_DIM,
                    _ => TEXT_LIGHT,
                };
                let status_icon = match s.status.as_str() {
                    "draft" => "○",
                    "ready" => "◎",
                    "in-progress" => "◉",
                    "shipped" => "●",
                    "archived" => "◌",
                    _ => "·",
                };
                ListItem::new(Line::from(vec![
                    Span::styled(
                        format!("{} ", status_icon),
                        Style::default().fg(status_color),
                    ),
                    Span::styled(
                        truncate_str(&s.title, 30),
                        Style::default().fg(TEXT_LIGHT),
                    ),
                    Span::raw(" "),
                    Span::styled(
                        s.area.as_deref().unwrap_or(""),
                        Style::default().fg(TEXT_DIM),
                    ),
                ]))
            })
            .collect();

        let list_block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(list_border_color));

        let mut list_state = self.list_state.clone();
        f.render_stateful_widget(
            List::new(items)
                .block(list_block)
                .highlight_style(Style::default().bg(PANEL_DARK).add_modifier(Modifier::BOLD))
                .highlight_symbol("▸ "),
            columns[0],
            &mut list_state,
        );

        // --- Right: Preview ---
        let preview_active = self.active_pane == SpecsPane::Preview;
        let preview_border = if preview_active {
            ACCENT_BLUE
        } else {
            Color::DarkGray
        };

        if let Some(spec) = &self.selected_spec {
            let mut lines = Vec::new();

            // Header info
            lines.push(Line::from(vec![
                Span::styled("Title:  ", Style::default().fg(TEXT_DIM)),
                Span::styled(&spec.title, Style::default().fg(TEXT_LIGHT).add_modifier(Modifier::BOLD)),
            ]));
            lines.push(Line::from(vec![
                Span::styled("Status: ", Style::default().fg(TEXT_DIM)),
                Span::styled(
                    spec.status.to_string(),
                    Style::default().fg(match spec.status {
                        SpecStatus::Draft => TEXT_DIM,
                        SpecStatus::Ready => ACCENT_BLUE,
                        SpecStatus::InProgress => ACCENT_YELLOW,
                        SpecStatus::Shipped => ACCENT_GREEN,
                        SpecStatus::Archived => TEXT_DIM,
                    }),
                ),
            ]));
            if let Some(ref area) = spec.area {
                lines.push(Line::from(vec![
                    Span::styled("Area:   ", Style::default().fg(TEXT_DIM)),
                    Span::styled(area, Style::default().fg(ACCENT_CYAN)),
                ]));
            }
            lines.push(Line::from(vec![
                Span::styled("Updated:", Style::default().fg(TEXT_DIM)),
                Span::styled(format!(" {}", spec.updated), Style::default().fg(TEXT_DIM)),
            ]));

            if !spec.open_questions.is_empty() {
                lines.push(Line::from(""));
                lines.push(Line::from(Span::styled(
                    "Open Questions:",
                    Style::default().fg(ACCENT_MAGENTA).add_modifier(Modifier::BOLD),
                )));
                for q in &spec.open_questions {
                    lines.push(Line::from(vec![
                        Span::styled("  • ", Style::default().fg(ACCENT_MAGENTA)),
                        Span::raw(q.as_str()),
                    ]));
                }
            }

            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                "───",
                Style::default().fg(Color::DarkGray),
            )));
            lines.push(Line::from(""));

            // Content
            for line in preview_content_to_lines(&spec.content) {
                lines.push(line);
            }

            let preview = Paragraph::new(Text::from(lines))
                .block(
                    Block::default()
                        .title(format!(" {} ", spec.slug))
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(preview_border)),
                )
                .wrap(Wrap { trim: false })
                .scroll((self.preview_scroll, 0));

            f.render_widget(preview, columns[1]);
        } else {
            let empty = Paragraph::new("Select a spec to preview")
                .style(Style::default().fg(TEXT_DIM))
                .block(
                    Block::default()
                        .title(" Preview ")
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(preview_border)),
                );
            f.render_widget(empty, columns[1]);
        }

        // --- Footer ---
        if self.search.active {
            self.search.render_footer(f, footer_area);
        } else {
            render_footer(
                f,
                footer_area,
                vec![
                    Span::styled(" n", Style::default().fg(ACCENT_CYAN).add_modifier(Modifier::BOLD)),
                    Span::raw(":new "),
                    Span::styled("e", Style::default().fg(ACCENT_CYAN).add_modifier(Modifier::BOLD)),
                    Span::raw(":edit "),
                    Span::styled("s", Style::default().fg(ACCENT_CYAN).add_modifier(Modifier::BOLD)),
                    Span::raw(":status "),
                    Span::styled("x", Style::default().fg(ACCENT_CYAN).add_modifier(Modifier::BOLD)),
                    Span::raw(":delete "),
                    Span::styled("/", Style::default().fg(ACCENT_CYAN).add_modifier(Modifier::BOLD)),
                    Span::raw(":search "),
                    Span::styled("Tab", Style::default().fg(ACCENT_CYAN).add_modifier(Modifier::BOLD)),
                    Span::raw(":switch "),
                    Span::styled("q", Style::default().fg(ACCENT_CYAN).add_modifier(Modifier::BOLD)),
                    Span::raw(":quit"),
                ],
            );
        }

        // --- Prompt Overlay ---
        match &self.prompt {
            AppPrompt::Input {
                label, buffer, ..
            } => {
                let area = centered_rect(50, 15, size);
                f.render_widget(Clear, area);
                let input = Paragraph::new(Line::from(vec![
                    Span::raw(buffer.as_str()),
                    Span::styled(
                        "█",
                        Style::default()
                            .fg(ACCENT_CYAN)
                            .add_modifier(Modifier::SLOW_BLINK),
                    ),
                ]))
                .block(
                    Block::default()
                        .title(format!(" {} ", label))
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(ACCENT_MAGENTA)),
                );
                f.render_widget(input, area);
            }
            AppPrompt::Select {
                label,
                options,
                selected,
                ..
            } => {
                let area = centered_rect(40, 30, size);
                f.render_widget(Clear, area);

                let items: Vec<ListItem> = options
                    .iter()
                    .enumerate()
                    .map(|(i, opt)| {
                        let style = if i == *selected {
                            Style::default()
                                .fg(ACCENT_CYAN)
                                .add_modifier(Modifier::BOLD)
                        } else {
                            Style::default().fg(TEXT_LIGHT)
                        };
                        let prefix = if i == *selected { "▸ " } else { "  " };
                        ListItem::new(Line::from(Span::styled(
                            format!("{}{}", prefix, opt),
                            style,
                        )))
                    })
                    .collect();

                f.render_widget(
                    List::new(items).block(
                        Block::default()
                            .title(format!(" {} ", label))
                            .borders(Borders::ALL)
                            .border_style(Style::default().fg(ACCENT_MAGENTA)),
                    ),
                    area,
                );
            }
            AppPrompt::None => {}
        }
    }
}

fn truncate_str(s: &str, max: usize) -> String {
    if s.len() > max {
        format!("{}...", &s[..max.saturating_sub(3)])
    } else {
        s.to_string()
    }
}
