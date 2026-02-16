use anyhow::{Result, Context};
use chisel_render::colors::*;
use chisel_docs::{Doc, ListOptions, DocsService};
use chisel_issues::{Issue, IssuesService, IssueStatus, IssuePriority};
use chisel_store::IssueRow;
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
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap, Clear},
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
    Input { label: String, buffer: String, kind: PromptKind },
    Select { label: String, options: Vec<String>, selected: usize, kind: PromptKind },
}

#[derive(Clone, Copy)]
enum PromptKind {
    NewDocTitle,
    MoveDocCategory,
    ReorderDocOrder,
    ReorderCategoryOrder,
    NewIssueTitle,
    EditIssueTitle,
    EditIssueLabels,
    EditIssuePriority,
    ChangeIssueStatus,
    ConfirmDeleteIssue,
    ReorderIssue,
}

fn preview_content_to_lines(content: &str) -> Vec<Line<'_>> {
    content.lines().map(|l| {
        if l.starts_with('#') {
            Line::from(Span::styled(l, Style::default().fg(ACCENT_YELLOW).add_modifier(Modifier::BOLD)))
        } else {
            Line::from(l)
        }
    }).collect()
}

fn render_footer(f: &mut Frame, area: Rect, spans: Vec<Span>) {
    f.render_widget(Clear, area);
    let footer = Paragraph::new(Line::from(spans)).style(Style::default().bg(PANEL_DARK));
    f.render_widget(footer, area);
}

fn next_list_item(state: &mut ListState, count: usize) {
    if count == 0 { return; }
    let i = match state.selected() {
        Some(i) => if i >= count - 1 { 0 } else { i + 1 },
        None => 0,
    };
    state.select(Some(i));
}

fn prev_list_item(state: &mut ListState, count: usize) {
    if count == 0 { return; }
    let i = match state.selected() {
        Some(i) => if i == 0 { count - 1 } else { i - 1 },
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
        render_footer(f, area, vec![
            Span::styled(" /", Style::default().fg(ACCENT_CYAN).add_modifier(Modifier::BOLD)),
            Span::raw(&self.buffer),
            Span::styled("█", Style::default().fg(ACCENT_CYAN).add_modifier(Modifier::SLOW_BLINK)),
        ]);
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
            let resolved = app.service.source.resolve_path(None, path.to_string_lossy().to_string()); // Simple resolve
            app.select_doc_by_path(resolved).await;
        } else {
            app.update_filtered_docs().await;
        }
        
        Ok(app)
    }

    async fn update_filtered_docs(&mut self) {
        let selected_cat = self.sidebar_state.selected()
            .and_then(|i| self.categories.get(i))
            .cloned()
            .unwrap_or_else(|| DisplayCategory { 
                order: -1, 
                label: "[ALL]".to_string(), 
                id: "[ALL]".to_string() 
            });

        let mut docs = if selected_cat.id == "[ALL]" {
            self.all_docs.clone()
        } else if selected_cat.id == "GENERAL" {
            self.all_docs.iter()
                .filter(|d| d.category.is_none())
                .cloned()
                .collect()
        } else {
            self.all_docs.iter()
                .filter(|d| d.category.as_deref() == Some(&selected_cat.id))
                .cloned()
                .collect()
        };

        if !self.search.buffer.is_empty() {
            let query = self.search.buffer.to_lowercase();
            docs.retain(|d| {
                d.name.to_lowercase().contains(&query) || 
                d.frontmatter.as_ref().map(|f| f.title.to_lowercase().contains(&query)).unwrap_or(false)
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
        execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
        terminal.show_cursor()?;

        res.map(|_| std::mem::replace(&mut self.exit_action, TuiAction::None))
    }

    async fn main_loop<B: ratatui::backend::Backend + io::Write>(&mut self, terminal: &mut Terminal<B>) -> Result<()> {
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
                            KeyCode::Char('[') | KeyCode::Char(']') => {
                                match self.active_pane {
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
                                                    buffer: doc.frontmatter.as_ref().and_then(|f| f.order).unwrap_or(0).to_string(),
                                                    kind: PromptKind::ReorderDocOrder,
                                                };
                                            }
                                        }
                                    }
                                    _ => {}
                                }
                            }
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
                            KeyCode::Char('j') | KeyCode::Down => {
                                match self.active_pane {
                                    DocsPane::Sidebar => { 
                                        next_list_item(&mut self.sidebar_state, self.categories.len());
                                        self.update_filtered_docs().await; 
                                    }
                                    DocsPane::Main => { 
                                        next_list_item(&mut self.main_list_state, self.filtered_docs.len());
                                        self.update_preview().await; 
                                    }
                                    DocsPane::Preview => { self.preview_scroll = self.preview_scroll.saturating_add(1); }
                                }
                            }
                            KeyCode::Char('k') | KeyCode::Up => {
                                match self.active_pane {
                                    DocsPane::Sidebar => { 
                                        prev_list_item(&mut self.sidebar_state, self.categories.len());
                                        self.update_filtered_docs().await; 
                                    }
                                    DocsPane::Main => { 
                                        prev_list_item(&mut self.main_list_state, self.filtered_docs.len());
                                        self.update_preview().await; 
                                    }
                                    DocsPane::Preview => { self.preview_scroll = self.preview_scroll.saturating_sub(1); }
                                }
                            }
                            KeyCode::Enter => {
                                if self.active_pane == DocsPane::Sidebar {
                                    self.active_pane = DocsPane::Main;
                                } else if let Some(selected) = self.main_list_state.selected() {
                                    if let Some(doc) = self.filtered_docs.get(selected) {
                                        disable_raw_mode()?;
                                        execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
                                        let _ = self.service.edit(doc.path.clone()).await;
                                        enable_raw_mode()?;
                                        execute!(terminal.backend_mut(), EnterAlternateScreen, EnableMouseCapture)?;
                                        terminal.clear()?;
                                        self.refresh_data().await?;
                                    }
                                }
                            }
                            KeyCode::Char('e') => {
                                if let Some(selected) = self.main_list_state.selected() {
                                    if let Some(doc) = self.filtered_docs.get(selected) {
                                        disable_raw_mode()?;
                                        execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
                                        let _ = self.service.edit(doc.path.clone()).await;
                                        enable_raw_mode()?;
                                        execute!(terminal.backend_mut(), EnterAlternateScreen, EnableMouseCapture)?;
                                        terminal.clear()?;
                                        self.refresh_data().await?;
                                    }
                                }
                            }
                            KeyCode::Char('n') => {
                                self.prompt = AppPrompt::Input { 
                                    label: "Document Title: ".to_string(), 
                                    buffer: String::new(), 
                                    kind: PromptKind::NewDocTitle 
                                };
                            }
                            KeyCode::Char('m') => {
                                if let Some(selected) = self.main_list_state.selected() {
                                    if let Some(doc) = self.filtered_docs.get(selected) {
                                        self.prompt = AppPrompt::Input { 
                                            label: "Category: ".to_string(), 
                                            buffer: doc.category.clone().unwrap_or_default(), 
                                            kind: PromptKind::MoveDocCategory 
                                        };
                                    }
                                }
                            }
                            _ => {}
                        }
                    } else {
                        // Handle Prompt Input
                        match &mut self.prompt {
                            AppPrompt::Input { buffer, kind, .. } => {
                                match key.code {
                                    KeyCode::Char(c) => buffer.push(c),
                                    KeyCode::Backspace => { buffer.pop(); },
                                    KeyCode::Esc => { self.prompt = AppPrompt::None; },
                                    KeyCode::Enter => {
                                        let input = buffer.clone();
                                        let kind = *kind;
                                        self.prompt = AppPrompt::None;
                                        self.handle_prompt_confirm(input, kind).await?;
                                    }
                                    _ => {}
                                }
                            },
                            AppPrompt::Select { selected, options, kind, .. } => {
                                match key.code {
                                    KeyCode::Char('j') | KeyCode::Down => {
                                        if *selected < options.len() - 1 { *selected += 1; }
                                    }
                                    KeyCode::Char('k') | KeyCode::Up => {
                                        if *selected > 0 { *selected -= 1; }
                                    }
                                    KeyCode::Esc => { self.prompt = AppPrompt::None; },
                                    KeyCode::Enter => {
                                        let idx = *selected;
                                        let kind = *kind;
                                        self.prompt = AppPrompt::None;
                                        self.handle_select_confirm(idx, kind).await?;
                                    }
                                    _ => {}
                                }
                            }
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
            },
            PromptKind::MoveDocCategory => {
                if let Some(selected) = self.main_list_state.selected() {
                    if let Some(doc) = self.filtered_docs.get(selected) {
                        let cat_opt = if input.is_empty() { None } else { Some(input) };
                        let _ = self.service.move_doc(doc.path.clone(), cat_opt).await?;
                        self.refresh_data().await?;
                    }
                }
            },
            PromptKind::ReorderDocOrder => {
                if let Ok(order) = input.parse::<i32>() {
                    if let Some(selected) = self.main_list_state.selected() {
                        if let Some(doc) = self.filtered_docs.get(selected) {
                            self.service.update_doc_order(doc.path.clone(), order).await?;
                            self.refresh_data().await?;
                        }
                    }
                }
            },
            PromptKind::ReorderCategoryOrder => {
                if let Ok(order) = input.parse::<i32>() {
                    if let Some(selected) = self.sidebar_state.selected() {
                        if let Some(cat) = self.categories.get(selected) {
                            self.service.update_category_order(&cat.id, order)?;
                            self.refresh_data().await?;
                        }
                    }
                }
            },
            _ => {}
        }
        Ok(())
    }

    async fn handle_select_confirm(&mut self, _idx: usize, _kind: PromptKind) -> Result<()> {
        Ok(())
    }

    async fn refresh_data(&mut self) -> Result<()> {
        let docs = self.service.list(ListOptions { root: self.service.workspace_root.clone(), use_gitignore: true, include_hidden: false }).await?;
        self.all_docs = docs.0;
        
        let mut categories_map = std::collections::BTreeMap::new();
        
        // Always include [ALL]
        categories_map.insert("[ALL]".to_string(), DisplayCategory {
            order: -1,
            label: "[ALL]".to_string(),
            id: "[ALL]".to_string(),
        });

        for doc in &self.all_docs {
            let cat_id = doc.category.as_deref().unwrap_or("GENERAL").to_string();
            if !categories_map.contains_key(&cat_id) {
                // Category metadata is still direct FS for now
                // Ideally we'd move this into DataSource
                let meta = chisel_docs::get_category_metadata(&self.service.workspace_root.join(".chisel").join("docs"), &cat_id);
                categories_map.insert(cat_id.clone(), DisplayCategory {
                    order: meta.order.unwrap_or(i32::MAX),
                    label: meta.label.unwrap_or_else(|| cat_id.clone()),
                    id: cat_id,
                });
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
        
        let chunks = Layout::default().direction(Direction::Vertical).constraints([Constraint::Min(0), Constraint::Length(1)]).split(f.area());
        let main_chunks = Layout::default().direction(Direction::Horizontal).constraints([Constraint::Percentage(20), Constraint::Percentage(30), Constraint::Percentage(50)]).split(chunks[0]);

        // Sidebar
        let sidebar_block = Block::default().borders(Borders::ALL).title(" CATEGORIES ").border_style(if self.active_pane == DocsPane::Sidebar { Style::default().fg(ACCENT_BLUE) } else { Style::default().fg(Color::DarkGray) });
        let sidebar_items: Vec<ListItem> = self.categories.iter().map(|cat| ListItem::new(Span::raw(&cat.label))).collect();
        f.render_stateful_widget(List::new(sidebar_items).block(sidebar_block).highlight_style(Style::default().bg(PANEL_DARK).add_modifier(Modifier::BOLD)).highlight_symbol("▸ "), main_chunks[0], &mut self.sidebar_state);

        // Main List
        let main_block = Block::default().borders(Borders::ALL).title(" DOCUMENTS ").border_style(if self.active_pane == DocsPane::Main { Style::default().fg(ACCENT_BLUE) } else { Style::default().fg(Color::DarkGray) });
        let main_items: Vec<ListItem> = self.filtered_docs.iter().map(|doc| ListItem::new(Span::raw(&doc.name))).collect();
        f.render_stateful_widget(List::new(main_items).block(main_block).highlight_style(Style::default().bg(PANEL_DARK).add_modifier(Modifier::BOLD)).highlight_symbol("▸ "), main_chunks[1], &mut self.main_list_state);

        // Preview
        let preview_chunks = Layout::default().direction(Direction::Vertical).constraints([Constraint::Min(0), Constraint::Length(8)]).split(main_chunks[2]);
        let preview_border_style = if self.active_pane == DocsPane::Preview { Style::default().fg(ACCENT_BLUE) } else { Style::default().fg(Color::DarkGray) };
        let content_text = self.selected_doc.as_ref().and_then(|d| d.content.as_deref()).unwrap_or("Select a document to preview");
        f.render_widget(Paragraph::new(preview_content_to_lines(content_text)).block(Block::default().borders(Borders::ALL).title(" CONTENT ").border_style(preview_border_style)).wrap(Wrap { trim: false }).scroll((self.preview_scroll, 0)), preview_chunks[0]);

        let metadata_block = Block::default().borders(Borders::ALL).title(" METADATA ").border_style(preview_border_style);
        if let Some(doc) = &self.selected_doc {
            let title = doc.frontmatter.as_ref().map(|f| f.title.as_str()).unwrap_or(&doc.name);
            let tags = doc.frontmatter.as_ref().map(|f| f.tags.join(", ")).unwrap_or_else(|| "none".to_string());
            let updated = doc.updated_at.format("%Y-%m-%d %H:%M").to_string();
            let metadata_text = Text::from(vec![
                Line::from(vec![Span::styled(" TITLE: ", Style::default().fg(TEXT_DIM)), Span::styled(title, Style::default().fg(ACCENT_BLUE).add_modifier(Modifier::BOLD))]),
                Line::from(vec![Span::styled(" TAGS:  ", Style::default().fg(TEXT_DIM)), Span::styled(tags, Style::default().fg(ACCENT_MAGENTA))]),
                Line::from(vec![Span::styled(" SYNC:  ", Style::default().fg(TEXT_DIM)), Span::styled(updated, Style::default().fg(ACCENT_GREEN))]),
            ]);
            f.render_widget(Paragraph::new(metadata_text).block(metadata_block), preview_chunks[1]);
        } else {
            f.render_widget(Paragraph::new("No metadata available").block(metadata_block), preview_chunks[1]);
        }

        match &self.prompt {
            AppPrompt::None => {
                if self.search.active {
                    self.search.render_footer(f, chunks[1]);
                } else {
                    render_footer(f, chunks[1], vec![
                        Span::styled(" q", Style::default().fg(ACCENT_RED).add_modifier(Modifier::BOLD)), Span::raw(":quit "),
                        Span::styled(" /", Style::default().fg(ACCENT_CYAN).add_modifier(Modifier::BOLD)), Span::raw(":find "),
                        Span::styled(" [ ]", Style::default().fg(ACCENT_YELLOW).add_modifier(Modifier::BOLD)), Span::raw(":reorder "),
                        Span::styled(" n", Style::default().fg(ACCENT_BLUE).add_modifier(Modifier::BOLD)), Span::raw(":new "),
                        Span::styled(" e", Style::default().fg(ACCENT_GREEN).add_modifier(Modifier::BOLD)), Span::raw(":edit "),
                        Span::styled(" m", Style::default().fg(ACCENT_MAGENTA).add_modifier(Modifier::BOLD)), Span::raw(":move "),
                        Span::styled(" Tab", Style::default().fg(ACCENT_CYAN).add_modifier(Modifier::BOLD)), Span::raw(":switch "),
                    ]);
                }
            },
            AppPrompt::Input { label, buffer, .. } => {
                render_footer(f, chunks[1], vec![
                    Span::styled(label, Style::default().fg(ACCENT_CYAN).add_modifier(Modifier::BOLD)),
                    Span::raw(buffer),
                    Span::styled("█", Style::default().fg(ACCENT_CYAN).add_modifier(Modifier::SLOW_BLINK)),
                ]);
            },
            AppPrompt::Select { label, options, selected, .. } => {
                render_footer(f, chunks[1], vec![Span::styled(" ESC", Style::default().fg(ACCENT_RED).add_modifier(Modifier::BOLD)), Span::raw(":cancel ")]);
                
                let area = centered_rect(40, 40, f.area());
                f.render_widget(Clear, area);
                let items: Vec<ListItem> = options.iter().enumerate().map(|(i, opt)| {
                    if i == *selected {
                        ListItem::new(Span::styled(format!("▸ {}", opt), Style::default().fg(ACCENT_BLUE).add_modifier(Modifier::BOLD)))
                    } else {
                        ListItem::new(Span::raw(format!("  {}", opt)))
                    }
                }).collect();
                f.render_widget(List::new(items).block(Block::default().borders(Borders::ALL).title(label.as_str()).border_style(Style::default().fg(ACCENT_MAGENTA))), area);
            }
        }
    }
}

// --- Issues Kanban App ---

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum IssuesPane {
    Todo,
    InProgress,
    Done,
    Preview,
}

pub struct IssuesApp {
    service: IssuesService,
    all_issues: Vec<IssueRow>,
    todo_list: Vec<IssueRow>,
    todo_state: ListState,
    inprogress_list: Vec<IssueRow>,
    inprogress_state: ListState,
    done_list: Vec<IssueRow>,
    done_state: ListState,
    selected_issue: Option<Issue>,
    active_pane: IssuesPane,
    preview_scroll: u16,
    prompt: AppPrompt,
    search: SearchState,
    exit_action: TuiAction,
    status_filter: Option<IssueStatus>,
}

impl IssuesApp {
    pub async fn new(service: IssuesService, status_filter: Option<IssueStatus>, initial_id: Option<i64>) -> Result<Self> {
        let mut app = Self {
            service,
            all_issues: Vec::new(),
            todo_list: Vec::new(),
            todo_state: ListState::default(),
            inprogress_list: Vec::new(),
            inprogress_state: ListState::default(),
            done_list: Vec::new(),
            done_state: ListState::default(),
            selected_issue: None,
            active_pane: IssuesPane::Todo,
            preview_scroll: 0,
            prompt: AppPrompt::None,
            search: SearchState::default(),
            exit_action: TuiAction::None,
            status_filter,
        };
        app.refresh_data().await?;
        app.todo_state.select(Some(0));
        
        if let Some(id) = initial_id {
            app.select_issue_by_id(id).await;
        } else if let Some(status) = &app.status_filter {
            app.active_pane = match status {
                IssueStatus::Todo => IssuesPane::Todo,
                IssueStatus::InProgress => IssuesPane::InProgress,
                _ => IssuesPane::Done,
            };
        }

        app.update_preview().await;
        Ok(app)
    }

    async fn select_issue_by_id(&mut self, id: i64) {
        if let Some(idx) = self.todo_list.iter().position(|i| i.id == id) {
            self.todo_state.select(Some(idx));
            self.active_pane = IssuesPane::Todo;
        } else if let Some(idx) = self.inprogress_list.iter().position(|i| i.id == id) {
            self.inprogress_state.select(Some(idx));
            self.active_pane = IssuesPane::InProgress;
        } else if let Some(idx) = self.done_list.iter().position(|i| i.id == id) {
            self.done_state.select(Some(idx));
            self.active_pane = IssuesPane::Done;
        }
    }

    async fn refresh_data(&mut self) -> Result<()> {
        let store = self.service.store.as_ref().context("Store not initialized")?;
        let status_str = self.status_filter.as_ref().map(|s| s.to_string());
        self.all_issues = store.get_issues(status_str.as_deref()).await?;
        self.update_filtered_issues();
        Ok(())
    }

    fn update_filtered_issues(&mut self) {
        let mut filtered = self.all_issues.clone();

        if !self.search.buffer.is_empty() {
            let query = self.search.buffer.to_lowercase();
            filtered.retain(|i| {
                i.title.to_lowercase().contains(&query) || 
                i.excerpt.to_lowercase().contains(&query) ||
                i.labels.as_ref().map(|l| l.to_lowercase().contains(&query)).unwrap_or(false)
            });
        }
        
        self.todo_list = filtered.iter().filter(|i| i.status == "todo").cloned().collect();
        self.inprogress_list = filtered.iter().filter(|i| i.status == "in-progress").cloned().collect();
        self.done_list = filtered.iter().filter(|i| i.status == "done" || i.status == "closed" || i.status == "cancelled").cloned().collect();
    }

    async fn update_preview(&mut self) {
        let id = match self.active_pane {
            IssuesPane::Todo => self.todo_state.selected().and_then(|i| self.todo_list.get(i)).map(|r| r.id),
            IssuesPane::InProgress => self.inprogress_state.selected().and_then(|i| self.inprogress_list.get(i)).map(|r| r.id),
            IssuesPane::Done => self.done_state.selected().and_then(|i| self.done_list.get(i)).map(|r| r.id),
            IssuesPane::Preview => None,
        };

        if let Some(id) = id {
            self.selected_issue = self.service.show(id).await.ok();
        } else if self.active_pane != IssuesPane::Preview {
            self.selected_issue = None;
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
        execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
        terminal.show_cursor()?;

        res.map(|_| std::mem::replace(&mut self.exit_action, TuiAction::None))
    }

    async fn main_loop<B: ratatui::backend::Backend + io::Write>(&mut self, terminal: &mut Terminal<B>) -> Result<()> {
        loop {
            terminal.draw(|f| self.render(f))?;

            if event::poll(std::time::Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    if self.search.handle_key(key) {
                        self.update_filtered_issues();
                        self.update_preview().await;
                    } else if let AppPrompt::None = self.prompt {
                        match key.code {
                            KeyCode::Char('q') | KeyCode::Esc => {
                                self.exit_action = TuiAction::Quit;
                                return Ok(());
                            }
                            KeyCode::Tab | KeyCode::Right | KeyCode::Char('l') => {
                                self.active_pane = match self.active_pane {
                                    IssuesPane::Todo => IssuesPane::InProgress,
                                    IssuesPane::InProgress => IssuesPane::Done,
                                    IssuesPane::Done => IssuesPane::Preview,
                                    IssuesPane::Preview => IssuesPane::Todo,
                                };
                                self.update_preview().await;
                            }
                            KeyCode::BackTab | KeyCode::Left | KeyCode::Char('h') => {
                                self.active_pane = match self.active_pane {
                                    IssuesPane::Todo => IssuesPane::Preview,
                                    IssuesPane::InProgress => IssuesPane::Todo,
                                    IssuesPane::Done => IssuesPane::InProgress,
                                    IssuesPane::Preview => IssuesPane::Done,
                                };
                                self.update_preview().await;
                            }
                            KeyCode::Char('j') | KeyCode::Down => {
                                match self.active_pane {
                                    IssuesPane::Todo => { next_list_item(&mut self.todo_state, self.todo_list.len()); self.update_preview().await; }
                                    IssuesPane::InProgress => { next_list_item(&mut self.inprogress_state, self.inprogress_list.len()); self.update_preview().await; }
                                    IssuesPane::Done => { next_list_item(&mut self.done_state, self.done_list.len()); self.update_preview().await; }
                                    IssuesPane::Preview => { self.preview_scroll = self.preview_scroll.saturating_add(1); }
                                }
                            }
                            KeyCode::Char('k') | KeyCode::Up => {
                                match self.active_pane {
                                    IssuesPane::Todo => { prev_list_item(&mut self.todo_state, self.todo_list.len()); self.update_preview().await; }
                                    IssuesPane::InProgress => { prev_list_item(&mut self.inprogress_state, self.inprogress_list.len()); self.update_preview().await; }
                                    IssuesPane::Done => { prev_list_item(&mut self.done_state, self.done_list.len()); self.update_preview().await; }
                                    IssuesPane::Preview => { self.preview_scroll = self.preview_scroll.saturating_sub(1); }
                                }
                            }
                            KeyCode::Char('e') => {
                                if let Some(issue) = &self.selected_issue {
                                    disable_raw_mode()?;
                                    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
                                    let _ = self.service.edit(issue.id).await;
                                    enable_raw_mode()?;
                                    execute!(terminal.backend_mut(), EnterAlternateScreen, EnableMouseCapture)?;
                                    terminal.clear()?;
                                    self.refresh_data().await?;
                                    self.update_preview().await;
                                }
                            }
                            KeyCode::Char('t') => {
                                if let Some(issue) = &self.selected_issue {
                                    self.prompt = AppPrompt::Input { 
                                        label: "Edit Title: ".to_string(), 
                                        buffer: issue.title.clone(), 
                                        kind: PromptKind::EditIssueTitle 
                                    };
                                }
                            }
                            KeyCode::Char('L') => {
                                if let Some(issue) = &self.selected_issue {
                                    self.prompt = AppPrompt::Input { 
                                        label: "Labels (comma separated): ".to_string(), 
                                        buffer: issue.labels.join(", "), 
                                        kind: PromptKind::EditIssueLabels 
                                    };
                                }
                            }
                            KeyCode::Char('p') => {
                                if self.selected_issue.is_some() {
                                    self.prompt = AppPrompt::Select { 
                                        label: " Select Priority ".to_string(), 
                                        options: vec!["Low".to_string(), "Medium".to_string(), "High".to_string(), "Critical".to_string()], 
                                        selected: 1, 
                                        kind: PromptKind::EditIssuePriority 
                                    };
                                }
                            }
                            KeyCode::Char('n') => {
                                self.prompt = AppPrompt::Input { 
                                    label: "Issue Title: ".to_string(), 
                                    buffer: String::new(), 
                                    kind: PromptKind::NewIssueTitle 
                                };
                            }
                            KeyCode::Char('m') => {
                                if self.selected_issue.is_some() {
                                    self.prompt = AppPrompt::Select { 
                                        label: " Select Status ".to_string(), 
                                        options: vec!["Todo".to_string(), "In Progress".to_string(), "Done".to_string(), "Closed".to_string(), "Cancelled".to_string()], 
                                        selected: 0, 
                                        kind: PromptKind::ChangeIssueStatus 
                                    };
                                }
                            }
                            KeyCode::Char('[') | KeyCode::Char(']') => {
                                if let Some(issue) = &self.selected_issue {
                                    self.prompt = AppPrompt::Input {
                                        label: format!("Order for #{}: ", issue.id),
                                        buffer: issue.order.to_string(),
                                        kind: PromptKind::ReorderIssue,
                                    };
                                }
                            }
                            KeyCode::Char('x') => {
                                if let Some(issue) = &self.selected_issue {
                                    self.prompt = AppPrompt::Input {
                                        label: format!(" Delete issue #{}? (y/n) ", issue.id),
                                        buffer: String::new(),
                                        kind: PromptKind::ConfirmDeleteIssue,
                                    };
                                }
                            }
                            _ => {}
                        }
                    } else {
                        // Handle Prompt Input
                        match &mut self.prompt {
                            AppPrompt::Input { buffer, kind, .. } => {
                                match key.code {
                                    KeyCode::Char(c) => buffer.push(c),
                                    KeyCode::Backspace => { buffer.pop(); },
                                    KeyCode::Esc => { self.prompt = AppPrompt::None; },
                                    KeyCode::Enter => {
                                        let input = buffer.clone();
                                        let kind = *kind;
                                        self.prompt = AppPrompt::None;
                                        self.handle_prompt_confirm(input, kind).await?;
                                    }
                                    _ => {}
                                }
                            },
                            AppPrompt::Select { selected, options, kind, .. } => {
                                match key.code {
                                    KeyCode::Char('j') | KeyCode::Down => {
                                        if *selected < options.len() - 1 { *selected += 1; }
                                    }
                                    KeyCode::Char('k') | KeyCode::Up => {
                                        if *selected > 0 { *selected -= 1; }
                                    }
                                    KeyCode::Esc => { self.prompt = AppPrompt::None; },
                                    KeyCode::Enter => {
                                        let idx = *selected;
                                        let kind = *kind;
                                        self.prompt = AppPrompt::None;
                                        self.handle_select_confirm(idx, kind).await?;
                                    }
                                    _ => {}
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
    }

    async fn handle_prompt_confirm(&mut self, input: String, kind: PromptKind) -> Result<()> {
        match kind {
            PromptKind::NewIssueTitle => {
                if !input.is_empty() {
                    self.service.create(&input, IssuePriority::Medium, Vec::new(), "Enter description...").await?;
                    self.refresh_data().await?;
                    self.update_preview().await;
                }
            },
            PromptKind::EditIssueTitle => {
                if let Some(issue) = &self.selected_issue {
                    if !input.is_empty() {
                        self.service.update_title(issue.id, input).await?;
                        self.refresh_data().await?;
                        self.update_preview().await;
                    }
                }
            },
            PromptKind::EditIssueLabels => {
                if let Some(issue) = &self.selected_issue {
                    let labels = input.split(',')
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                        .collect();
                    self.service.update_labels(issue.id, labels).await?;
                    self.refresh_data().await?;
                    self.update_preview().await;
                }
            },
            PromptKind::ConfirmDeleteIssue => {
                if input.to_lowercase() == "y" {
                    if let Some(issue) = &self.selected_issue {
                        self.service.delete(issue.id).await?;
                        self.refresh_data().await?;
                        self.update_preview().await;
                    }
                }
            },
            PromptKind::ReorderIssue => {
                if let Ok(order) = input.parse::<i32>() {
                    if let Some(issue) = &self.selected_issue {
                        self.service.update_order(issue.id, order).await?;
                        self.refresh_data().await?;
                        self.update_preview().await;
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }

    async fn handle_select_confirm(&mut self, idx: usize, kind: PromptKind) -> Result<()> {
        match kind {
            PromptKind::ChangeIssueStatus => {
                if let Some(issue) = &self.selected_issue {
                    let status = match idx {
                        0 => IssueStatus::Todo,
                        1 => IssueStatus::InProgress,
                        2 => IssueStatus::Done,
                        3 => IssueStatus::Closed,
                        4 => IssueStatus::Cancelled,
                        _ => IssueStatus::Todo,
                    };
                    self.service.update_status(issue.id, status).await?;
                    self.refresh_data().await?;
                    self.update_preview().await;
                }
            },
            PromptKind::EditIssuePriority => {
                if let Some(issue) = &self.selected_issue {
                    let priority = match idx {
                        0 => IssuePriority::Low,
                        1 => IssuePriority::Medium,
                        2 => IssuePriority::High,
                        3 => IssuePriority::Critical,
                        _ => IssuePriority::Medium,
                    };
                    self.service.update_priority(issue.id, priority).await?;
                    self.refresh_data().await?;
                    self.update_preview().await;
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn render(&mut self, f: &mut Frame) {
        f.render_widget(Clear, f.area());

        let chunks = Layout::default().direction(Direction::Vertical).constraints([Constraint::Min(0), Constraint::Length(1)]).split(f.area());
        let board_chunks = Layout::default().direction(Direction::Horizontal).constraints([Constraint::Percentage(60), Constraint::Percentage(40)]).split(chunks[0]);
        let column_chunks = Layout::default().direction(Direction::Horizontal).constraints([Constraint::Percentage(33), Constraint::Percentage(33), Constraint::Percentage(34)]).split(board_chunks[0]);

        render_issue_column(f, column_chunks[0], " TODO ", &self.todo_list, &mut self.todo_state, self.active_pane == IssuesPane::Todo);
        render_issue_column(f, column_chunks[1], " IN PROGRESS ", &self.inprogress_list, &mut self.inprogress_state, self.active_pane == IssuesPane::InProgress);
        render_issue_column(f, column_chunks[2], " DONE ", &self.done_list, &mut self.done_state, self.active_pane == IssuesPane::Done);

        let preview_chunks = Layout::default().direction(Direction::Vertical).constraints([Constraint::Min(0), Constraint::Length(10)]).split(board_chunks[1]);
        let preview_border_style = if self.active_pane == IssuesPane::Preview { Style::default().fg(ACCENT_BLUE) } else { Style::default().fg(Color::DarkGray) };
        let content_text = self.selected_issue.as_ref().map(|i| i.content.as_str()).unwrap_or("Select an issue to preview");
        f.render_widget(Paragraph::new(preview_content_to_lines(content_text)).block(Block::default().borders(Borders::ALL).title(" DESCRIPTION ").border_style(preview_border_style)).wrap(Wrap { trim: false }).scroll((self.preview_scroll, 0)), preview_chunks[0]);

        let metadata_block = Block::default().borders(Borders::ALL).title(" DETAILS ").border_style(preview_border_style);
        if let Some(issue) = &self.selected_issue {
            let metadata_text = Text::from(vec![
                Line::from(vec![Span::styled(" ID:       ", Style::default().fg(TEXT_DIM)), Span::styled(format!("#{}", issue.id), Style::default().fg(ACCENT_BLUE).add_modifier(Modifier::BOLD))]),
                Line::from(vec![Span::styled(" PRIORITY: ", Style::default().fg(TEXT_DIM)), Span::styled(format!("{:?}", issue.priority), Style::default().fg(ACCENT_RED))]),
                Line::from(vec![Span::styled(" LABELS:   ", Style::default().fg(TEXT_DIM)), Span::styled(issue.labels.join(", "), Style::default().fg(ACCENT_MAGENTA))]),
            ]);
            f.render_widget(Paragraph::new(metadata_text).block(metadata_block), preview_chunks[1]);
        } else {
            f.render_widget(Paragraph::new("No details available").block(metadata_block), preview_chunks[1]);
        }

        match &self.prompt {
            AppPrompt::None => {
                if self.search.active {
                    self.search.render_footer(f, chunks[1]);
                } else {
                    render_footer(f, chunks[1], vec![
                        Span::styled(" q", Style::default().fg(ACCENT_RED).add_modifier(Modifier::BOLD)), Span::raw(":quit "),
                        Span::styled(" /", Style::default().fg(ACCENT_CYAN).add_modifier(Modifier::BOLD)), Span::raw(":find "),
                        Span::styled(" [ ]", Style::default().fg(ACCENT_YELLOW).add_modifier(Modifier::BOLD)), Span::raw(":reorder "),
                        Span::styled(" n", Style::default().fg(ACCENT_BLUE).add_modifier(Modifier::BOLD)), Span::raw(":new "),
                        Span::styled(" e", Style::default().fg(ACCENT_GREEN).add_modifier(Modifier::BOLD)), Span::raw(":edit "),
                        Span::styled(" t", Style::default().fg(ACCENT_YELLOW).add_modifier(Modifier::BOLD)), Span::raw(":title "),
                        Span::styled(" L", Style::default().fg(ACCENT_MAGENTA).add_modifier(Modifier::BOLD)), Span::raw(":labels "),
                        Span::styled(" p", Style::default().fg(ACCENT_RED).add_modifier(Modifier::BOLD)), Span::raw(":prio "),
                        Span::styled(" m", Style::default().fg(ACCENT_MAGENTA).add_modifier(Modifier::BOLD)), Span::raw(":status "),
                        Span::styled(" x", Style::default().fg(ACCENT_RED).add_modifier(Modifier::BOLD)), Span::raw(":del "),
                        Span::styled(" Tab", Style::default().fg(ACCENT_CYAN).add_modifier(Modifier::BOLD)), Span::raw(":switch "),
                    ]);
                }
            },
            AppPrompt::Input { label, buffer, .. } => {
                render_footer(f, chunks[1], vec![
                    Span::styled(label, Style::default().fg(ACCENT_CYAN).add_modifier(Modifier::BOLD)),
                    Span::raw(buffer),
                    Span::styled("█", Style::default().fg(ACCENT_CYAN).add_modifier(Modifier::SLOW_BLINK)),
                ]);
            },
            AppPrompt::Select { label, options, selected, .. } => {
                render_footer(f, chunks[1], vec![Span::styled(" ESC", Style::default().fg(ACCENT_RED).add_modifier(Modifier::BOLD)), Span::raw(":cancel ")]);
                
                let area = centered_rect(40, 40, f.area());
                f.render_widget(Clear, area);
                let items: Vec<ListItem> = options.iter().enumerate().map(|(i, opt)| {
                    if i == *selected {
                        ListItem::new(Span::styled(format!("▸ {}", opt), Style::default().fg(ACCENT_BLUE).add_modifier(Modifier::BOLD)))
                    } else {
                        ListItem::new(Span::raw(format!("  {}", opt)))
                    }
                }).collect();
                f.render_widget(List::new(items).block(Block::default().borders(Borders::ALL).title(label.as_str()).border_style(Style::default().fg(ACCENT_MAGENTA))), area);
            }
        }
    }
}

fn render_issue_column(f: &mut Frame, area: Rect, title: &str, list: &[IssueRow], state: &mut ListState, focused: bool) {
    let block = Block::default().borders(Borders::ALL).title(title).border_style(if focused { Style::default().fg(ACCENT_BLUE) } else { Style::default().fg(Color::DarkGray) });
    let items: Vec<ListItem> = list.iter().map(|i| {
        let priority_color = match i.priority.as_str() {
            "critical" | "high" => ACCENT_RED,
            "medium" => ACCENT_YELLOW,
            _ => ACCENT_GREEN,
        };
        ListItem::new(vec![
            Line::from(vec![
                Span::styled(format!("#{} ", i.id), Style::default().fg(TEXT_DIM)),
                Span::raw(&i.title),
            ]),
            Line::from(vec![
                Span::styled(format!("  {}", i.priority), Style::default().fg(priority_color)),
            ]),
        ])
    }).collect();
    f.render_stateful_widget(List::new(items).block(block).highlight_style(Style::default().bg(PANEL_DARK).add_modifier(Modifier::BOLD)).highlight_symbol("▸ "), area, state);
}
