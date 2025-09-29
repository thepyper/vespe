use ratatui::{prelude::*, widgets::*};
use crossterm::event::KeyCode;
use crate::{App, MessageType};
use vespe::Agent;
use tracing::{info, error};

#[derive(Debug)]
pub struct AgentsPageState {
    pub agents: Vec<Agent>,
    pub selected_agent_index: usize,
    pub last_selected_agent_uid: Option<String>,
}

impl Default for AgentsPageState {
    fn default() -> Self {
        Self {
            agents: Vec::new(),
            selected_agent_index: 0,
            last_selected_agent_uid: None,
        }
    }
}

pub fn render_agents_page(frame: &mut Frame, area: Rect, state: &AgentsPageState) {
    let agents_items: Vec<ListItem> = state.agents
        .iter()
        .enumerate()
        .map(|(i, agent)| {
            let agent_type_str = match agent.details {
                vespe::AgentDetails::AI(_) => "AI",
                vespe::AgentDetails::Human(_) => "Human",
            };
            let content = format!("{} - {} ({})", agent.metadata.uid, agent.metadata.name, agent_type_str);
            if i == state.selected_agent_index {
                ListItem::new(content).style(Style::default().fg(Color::Black).bg(Color::LightMagenta))
            } else {
                ListItem::new(content)
            }
        })
        .collect();

    let agents_list = List::new(agents_items)
        .block(Block::default().borders(Borders::ALL).title("Agents"))
        .highlight_style(Style::default().add_modifier(Modifier::BOLD))
        .highlight_symbol("> ");

    frame.render_widget(agents_list, area);
}

pub fn handle_events(app: &mut App, key_code: KeyCode) -> Result<(), anyhow::Error> {
    match key_code {
        KeyCode::Up => {
            info!("Agents: KeyCode::Up pressed.");
            if !app.agents_page_state.agents.is_empty() {
                if app.agents_page_state.selected_agent_index > 0 {
                    app.agents_page_state.selected_agent_index -= 1;
                    app.agents_page_state.last_selected_agent_uid = Some(app.agents_page_state.agents[app.agents_page_state.selected_agent_index].metadata.uid.clone());
                    info!("Agents: Selected agent index: {}.", app.agents_page_state.selected_agent_index);
                }
            }
        }
        KeyCode::Down => {
            info!("Agents: KeyCode::Down pressed.");
            if !app.agents_page_state.agents.is_empty() {
                if app.agents_page_state.selected_agent_index < app.agents_page_state.agents.len() - 1 {
                    app.agents_page_state.selected_agent_index += 1;
                    app.agents_page_state.last_selected_agent_uid = Some(app.agents_page_state.agents[app.agents_page_state.selected_agent_index].metadata.uid.clone());
                    info!("Agents: Selected agent index: {}.", app.agents_page_state.selected_agent_index);
                }
            }
        }
        _ => {},
    }
    Ok(())
}

pub fn load_agents_into_state(app: &mut App) -> Result<(), anyhow::Error> {
    match app.project.list_agents() {
        Ok(agents) => {
            app.agents_page_state.agents = agents;

            // Try to restore the last selected agent
            if let Some(last_uid) = &app.agents_page_state.last_selected_agent_uid {
                if let Some(index) = app.agents_page_state.agents.iter().position(|a| &a.metadata.uid == last_uid) {
                    app.agents_page_state.selected_agent_index = index;
                } else if !app.agents_page_state.agents.is_empty() {
                    // If the last selected agent is no longer available, select the first one
                    app.agents_page_state.selected_agent_index = 0;
                } else {
                    // No agents available
                    app.agents_page_state.selected_agent_index = 0;
                }
            } else if !app.agents_page_state.agents.is_empty() {
                // If no last selected UID, but agents are available, select the first one
                app.agents_page_state.selected_agent_index = 0;
            } else {
                // No agents available
                app.agents_page_state.selected_agent_index = 0;
            }
            info!("Agents loaded successfully.");
        }
        Err(e) => {
            app.message = Some(format!("Error loading agents: {:?}", e));
            app.message_type = MessageType::Error;
            error!("Error loading agents: {:?}", e);
        }
    }
    Ok(())
}