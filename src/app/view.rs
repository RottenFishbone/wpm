use std::{collections::VecDeque, iter::FromIterator};

use super::{Model, RoundState};
use tui::{
    Frame, 
    backend::Backend, 
    layout::{Constraint, Layout, Direction, Corner, Rect, Alignment}, 
    style::{Modifier, Style, Color}, 
    text::{Spans, Span}, 
    widgets::{Block, List, ListItem, Borders, Paragraph, Wrap}
};

pub fn render<B: Backend>(f: &mut Frame<B>, model: &Model) {
    let padding_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(25),
                      Constraint::Percentage(50),
                      Constraint::Percentage(25)].as_ref())
        .split(f.size());
    
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50),
                      Constraint::Length(3),
                      Constraint::Length(3),
                      Constraint::Min(0)].as_ref())
        .split(padding_chunks[1]);
    

    let lower_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(30),
                      Constraint::Percentage(66)])
        .split(main_chunks[2]);

    draw_words_list(f, main_chunks[1], model);
    draw_current_word(f, lower_chunks[0], model);
    draw_info(f, lower_chunks[1], model);
}

fn draw_info<B: Backend>(f: &mut Frame<B>, area: Rect, model: &Model) {
    let timer_span: Span;

    let elapsed = model.start.elapsed().unwrap().as_secs();
    match model.round_state {
        RoundState::Active => {
            timer_span = Span::from(format!(
                                        "{}s | ~{} wpm", 
                                        30-elapsed,
                                        model.chars_correct/5*2)); 
        },
        RoundState::Stopped => {
            timer_span = Span::from("---");
        },
        RoundState::Completed => {
            let (correct, wrong) = (model.chars_correct, model.chars_wrong);
            let total = correct+wrong;
            let accuracy: f64 = (correct as f64) / (total as f64);
            let gross_words: f64 = (total as f64) / 5.0;
            let gross_wpm: f64 = gross_words * 2.0;
            let adjusted_wpm: f64 = gross_wpm * accuracy;
            timer_span = Span::from(format!("{} wpm", adjusted_wpm as u64));
        },
    }

    let text = Paragraph::new(timer_span)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(text, area);
}

fn draw_words_list<B: Backend>(f: &mut Frame<B>, area: Rect, model: &Model){
    let mut next_words = model.word_queue.iter().take(10);
    let mut words: Vec<Span> = Vec::new();
    while let Some(word) = next_words.next() {
        if words.is_empty() {
            let mut green_chars = Vec::<char>::new();
            let mut default_chars = Vec::<char>::new();

            let mut target_word = VecDeque::from_iter(word.chars());
            let mut typed_word=VecDeque::from_iter(model.word_typing.chars());

            // Loop through all the chars of the target word
            while let Some(tar_char) = target_word.pop_front(){
                // Compare typed char with target char
                if let Some(typed_char) = typed_word.pop_front() {
                    if typed_char == tar_char {
                        green_chars.push(typed_char);
                    }
                    else{
                        // Push the entire word as a red style
                        let style = Style::default().fg(Color::Red);
                        words.push(Span::styled(word, style));

                        // Clear the other styled entries
                        green_chars.clear();
                        default_chars.clear();
                        
                        // Break and push to be drawn
                        break;
                    }
                }
                // End of currently typed word, leave the rest as the default
                // style
                else{
                    // Push the popped char back to the word
                    target_word.push_front(tar_char);
                    // Build a Vec<char> from the VecDeque
                    default_chars = Vec::from_iter(
                                        target_word.iter()
                                                   .map(|c| c.clone()));
                    // Break and push to be drawn
                    break;
                }
            }

            let green_string: String = green_chars.into_iter().collect(); 
            words.push(Span::styled(green_string,
                                    Style::default().fg(Color::Green)));
            let default_string: String = default_chars.into_iter().collect(); 
            words.push(Span::from(default_string));
        }
        else{
            words.push(Span::from(String::from(word)));
        } 

        words.push(Span::from(" "));

        
    }

    let text_box = Paragraph::new(Spans::from(words))
        .block(Block::default().borders(Borders::ALL))
        .wrap(Wrap { trim: true });

    f.render_widget(text_box, area);
}

fn draw_current_word<B: Backend>(f: &mut Frame<B>, area: Rect, model: &Model){
    let padding_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(30),
                      Constraint::Percentage(66)])
        .split(area);

    let block = Block::default()
        .borders(Borders::ALL);
    let text_box = Paragraph::new(model.word_typing.clone())
        .block(block)
        .wrap(Wrap { trim: true });

    f.render_widget(text_box, padding_chunks[0]);
}
